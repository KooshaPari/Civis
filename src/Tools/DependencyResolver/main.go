package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"log"
	"os"
)

// PackManifest mirrors the C# struct
type PackManifest struct {
	ID             string   `json:"id"`
	Name           string   `json:"name"`
	Version        string   `json:"version"`
	FrameworkVersion string `json:"framework_version"`
	DependsOn      []string `json:"depends_on"`
	ConflictsWith  []string `json:"conflicts_with"`
	LoadOrder      int      `json:"load_order"`
}

// ResolverInput matches C# input JSON format
type ResolverInput struct {
	Available []PackManifest `json:"available"`
	Target    PackManifest   `json:"target"`
}

// ResolverOutput matches C# output JSON format
type ResolverOutput struct {
	Resolved []string `json:"resolved"` // Pack IDs in load order
	Errors   []string `json:"errors"`
}

// PackResolver implements Kahn's algorithm for topological sort
type PackResolver struct {
	packByID   map[string]*PackManifest
	inDegree   map[string]int
	dependents map[string][]string
}

// NewPackResolver creates a fresh resolver
func NewPackResolver(available []*PackManifest) *PackResolver {
	packByID := make(map[string]*PackManifest)
	for _, p := range available {
		packByID[p.ID] = p
	}

	inDegree := make(map[string]int)
	dependents := make(map[string][]string)

	// Initialize maps
	for _, p := range available {
		inDegree[p.ID] = 0
		dependents[p.ID] = []string{}
	}

	return &PackResolver{
		packByID:   packByID,
		inDegree:   inDegree,
		dependents: dependents,
	}
}

// Resolve computes topological load order using Kahn's algorithm
func (r *PackResolver) Resolve(target *PackManifest) ([]string, []string) {
	errors := []string{}

	// Step 1: Check all dependencies exist
	for _, dep := range target.DependsOn {
		if _, ok := r.packByID[dep]; !ok {
			errors = append(errors, fmt.Sprintf("Pack '%s' requires missing dependency: '%s'", target.ID, dep))
		}
	}

	if len(errors) > 0 {
		return nil, errors
	}

	// Step 2: Build in-degree map and adjacency list (dependency graph)
	for _, pack := range r.packByID {
		for _, dep := range pack.DependsOn {
			if _, ok := r.packByID[dep]; ok {
				// dep must come before pack, so pack depends on dep
				r.dependents[dep] = append(r.dependents[dep], pack.ID)
				r.inDegree[pack.ID]++
			} else {
				errors = append(errors, fmt.Sprintf("Pack '%s' depends on unknown pack '%s'", pack.ID, dep))
			}
		}
	}

	if len(errors) > 0 {
		return nil, errors
	}

	// Step 3: Kahn's algorithm using ready queue (sorted by LoadOrder)
	ready := []*PackManifest{}

	// Find all packs with in-degree 0
	for _, pack := range r.packByID {
		if r.inDegree[pack.ID] == 0 {
			ready = append(ready, pack)
		}
	}

	// Sort by LoadOrder (tiebreaker)
	for i := 0; i < len(ready); i++ {
		for j := i + 1; j < len(ready); j++ {
			if ready[j].LoadOrder < ready[i].LoadOrder {
				ready[i], ready[j] = ready[j], ready[i]
			}
		}
	}

	sorted := []string{}
	processedCount := 0

	for len(ready) > 0 {
		// Pop first (lowest LoadOrder)
		current := ready[0]
		ready = ready[1:]

		sorted = append(sorted, current.ID)
		processedCount++

		// Reduce in-degree for all dependents
		for _, depID := range r.dependents[current.ID] {
			r.inDegree[depID]--
			if r.inDegree[depID] == 0 {
				// Insert in sorted order
				depPack := r.packByID[depID]
				inserted := false
				for i, p := range ready {
					if depPack.LoadOrder < p.LoadOrder {
						ready = append(ready[:i], append([]*PackManifest{depPack}, ready[i:]...)...)
						inserted = true
						break
					}
				}
				if !inserted {
					ready = append(ready, depPack)
				}
			}
		}
	}

	// Step 4: Check for cycles
	if processedCount != len(r.packByID) {
		errors = append(errors, "Circular dependency detected among packs")
		return nil, errors
	}

	return sorted, nil
}

func main() {
	inputPath := flag.String("input", "", "Input JSON file path")
	outputPath := flag.String("output", "", "Output JSON file path")
	flag.Parse()

	if *inputPath == "" || *outputPath == "" {
		fmt.Fprintf(os.Stderr, "Usage: dinoforge-resolver --input <file> --output <file>\n")
		os.Exit(1)
	}

	// Step 1: Read input JSON
	inputData, err := os.ReadFile(*inputPath)
	if err != nil {
		log.Fatalf("Failed to read input: %v", err)
	}

	var input ResolverInput
	if err := json.Unmarshal(inputData, &input); err != nil {
		log.Fatalf("Failed to parse input JSON: %v", err)
	}

	// Step 2: Build pack pointers (for resolver)
	availablePtrs := make([]*PackManifest, len(input.Available))
	for i := range input.Available {
		availablePtrs[i] = &input.Available[i]
	}

	// Step 3: Resolve dependencies
	resolver := NewPackResolver(availablePtrs)
	resolved, errors := resolver.Resolve(&input.Target)

	output := ResolverOutput{
		Resolved: resolved,
		Errors:   errors,
	}

	// Step 4: Write output JSON
	outputJSON, err := json.Marshal(output)
	if err != nil {
		log.Fatalf("Failed to marshal output: %v", err)
	}

	if err := os.WriteFile(*outputPath, outputJSON, 0644); err != nil {
		log.Fatalf("Failed to write output: %v", err)
	}

	// Exit with appropriate code
	if len(errors) > 0 {
		os.Exit(1)
	}
	os.Exit(0)
}
