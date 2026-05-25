#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const https = require('https');

// Helper to calculate confidence score
function calculateConfidence(model) {
    let confidence = 0.5;
    const faces = model.faceCount || 0;
    const title = (model.name || '').toLowerCase();

    // Polycount scoring
    if (faces >= 200 && faces <= 1500) {
        confidence = 0.95;
    } else if (faces >= 100 && faces <= 5000) {
        confidence = 0.85;
    } else if (faces > 5000 && faces <= 20000) {
        confidence = 0.70;
    }

    // Theme boost
    const themeKeywords = ['desert', 'tatooine', 'outer', 'rim', 'scrap', 'settlement', 'cantina', 'moisture', 'dune'];
    for (const kw of themeKeywords) {
        if (title.includes(kw)) {
            confidence = Math.min(1.0, confidence + 0.15);
            break;
        }
    }

    return Math.round(confidence * 100) / 100;
}

// Extract model data
function extractModel(model) {
    const thumbnails = model.thumbnails?.images || [];
    const previewUrl = thumbnails.length > 0 ? thumbnails[0].url : null;

    return {
        model_id: model.uid,
        title: model.name,
        author: model.user?.username,
        url: `https://sketchfab.com/models/${model.uid}`,
        license: model.license?.label || 'Not Specified',
        polycount: {
            vertices: model.vertexCount || 0,
            faces: model.faceCount || 0,
            estimated_triangles: model.faceCount || 0
        },
        preview_image_url: previewUrl,
        description: model.description || '',
        is_downloadable: model.isDownloadable || false,
        is_pack: (model.name || '').toLowerCase().includes('pack') || (model.name || '').toLowerCase().includes('collection'),
        confidence_score: calculateConfidence(model),
        download_formats: ['glb', 'obj', 'fbx', 'usdz'],
        view_count: model.viewCount || 0,
        published_at: model.publishedAt || ''
    };
}

// Main
const queries = [
    { name: 'tatooine_buildings', url: 'https://api.sketchfab.com/v3/search?type=models&q=tatooine+buildings&count=20&sort_by=-relevance' },
    { name: 'star_wars_buildings', url: 'https://api.sketchfab.com/v3/search?type=models&q=star+wars+buildings&count=20&sort_by=-relevance' },
    { name: 'low_poly_star_wars', url: 'https://api.sketchfab.com/v3/search?type=models&q=low+poly+star+wars&count=20&sort_by=-relevance' }
];

const allModels = [];
const seenIds = new Set();

// Ensure directory exists
if (!fs.existsSync('.claude/sw_search')) {
    fs.mkdirSync('.claude/sw_search', { recursive: true });
}

function fetchQuery(queryData, callback) {
    https.get(queryData.url, (res) => {
        let data = '';
        res.on('data', chunk => data += chunk);
        res.on('end', () => {
            try {
                const json = JSON.parse(data);
                const results = json.results || [];
                const filepath = path.join('.claude/sw_search', `${queryData.name}.json`);
                fs.writeFileSync(filepath, JSON.stringify(json, null, 2));

                console.error(`Loaded ${results.length} models from ${queryData.name}`);

                for (const model of results.slice(0, 10)) {
                    if (seenIds.has(model.uid)) continue;
                    seenIds.add(model.uid);

                    const extracted = extractModel(model);
                    extracted.source_query = queryData.name;
                    allModels.push(extracted);
                }

                callback();
            } catch (e) {
                console.error('Parse error:', e.message);
                callback();
            }
        });
    }).on('error', (e) => {
        console.error('Fetch error:', e.message);
        callback();
    });
}

// Sequential fetch
let idx = 0;
function processNext() {
    if (idx >= queries.length) {
        finalize();
    } else {
        fetchQuery(queries[idx], () => {
            idx++;
            // Small delay between requests
            setTimeout(processNext, 500);
        });
    }
}

function finalize() {
    // Sort by confidence
    allModels.sort((a, b) => b.confidence_score - a.confidence_score);

    // Select top 3
    const top3 = [];
    for (const model of allModels) {
        if (top3.length >= 3) break;

        const tri = model.polycount.estimated_triangles;

        // Prefer 200-10,000 triangle range
        if ((tri >= 200 && tri <= 10000) || model.confidence_score >= 0.85) {
            top3.push(model);
        }
    }

    // Output
    const output = {
        timestamp: new Date().toISOString(),
        search_queries: [
            'tatooine_buildings',
            'star_wars_buildings',
            'low_poly_star_wars'
        ],
        total_candidates_found: allModels.length,
        top_3_candidates: top3.slice(0, 3),
        scoring_criteria: {
            polycount_range_ideal: '200-1000 triangles',
            polycount_range_acceptable: '100-20000 triangles',
            themes_preferred: ['desert', 'tatooine', 'outer-rim', 'scrap', 'settlement'],
            licenses_preferred: ['CC-BY', 'CC0', 'Not Specified (check download)'],
            format_preference: ['glb', 'fbx', 'obj']
        },
        notes: 'NOT downloaded yet - metadata only collected for evaluation'
    };

    console.log(JSON.stringify(output, null, 2));

    // Also save to file
    fs.writeFileSync(path.join('.claude/sw_search', 'candidates.json'), JSON.stringify(output, null, 2));
}

processNext();
