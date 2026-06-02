using System.Text;
using System.Text.Json;
using AssetsTools.NET;
using AssetsTools.NET.Extra;

// SkeletonExtractor (#991): read a DINO Addressables bundle, find SkinnedMeshRenderer
// meshes (units), and dump bone names + bindpose count. Used to reverse-engineer the
// DINO infantry reference skeleton (e.g. dark_knight, 21 bindposes) so SW meshes can be
// retargeted to a matching bone count.
//
// Usage:
//   skeleton-extractor <bundle-path> [name-filter] [out-json]
// Example:
//   skeleton-extractor ".../defaultlocalgroup_assets_all.bundle" dark_knight skeleton.json

if (args.Length < 1)
{
    Console.Error.WriteLine("usage: skeleton-extractor <bundle> [name-filter] [out-json]");
    return 1;
}

string bundlePath = args[0];
string? filter = args.Length > 1 && !string.IsNullOrEmpty(args[1]) ? args[1] : null;
string? outJson = args.Length > 2 ? args[2] : null;
// "smr" mode: resolve SkinnedMeshRenderer bone Transform names (true hierarchy).
bool smrMode = Environment.GetEnvironmentVariable("SMR") == "1";

if (!File.Exists(bundlePath))
{
    Console.Error.WriteLine($"bundle not found: {bundlePath}");
    return 1;
}

var am = new AssetsManager();
// Load class package if present (needed when bundle lacks type trees).
foreach (string tpk in new[] { "classdata.tpk", "lz4.tpk" })
{
    string tpkPath = Path.Combine(AppContext.BaseDirectory, tpk);
    if (File.Exists(tpkPath))
    {
        try { am.LoadClassPackage(tpkPath); Console.Error.WriteLine($"[info] loaded class package {tpk}"); }
        catch (Exception ex) { Console.Error.WriteLine($"[warn] tpk load failed: {ex.Message}"); }
        break;
    }
}
Console.Error.WriteLine($"[info] loading bundle (may take a while for large files): {bundlePath}");
BundleFileInstance bun = am.LoadBundleFile(bundlePath, true);
Console.Error.WriteLine($"[info] bundle loaded, {bun.file.GetAllFileNames().Count()} directories");

var results = new List<object>();
int meshHits = 0;

for (int dirIdx = 0; dirIdx < bun.file.GetAllFileNames().Count(); dirIdx++)
{
    AssetsFileInstance afile;
    try
    {
        afile = am.LoadAssetsFileFromBundle(bun, dirIdx, true);
    }
    catch (Exception ex)
    {
        Console.Error.WriteLine($"[warn] dir {dirIdx} load failed: {ex.Message}");
        continue;
    }

    // Ensure the class database / type tree is available for deserialization.
    string uver;
    try { uver = afile.file.Metadata?.UnityVersion ?? "2021.3.45f2"; }
    catch { Console.Error.WriteLine($"[warn] dir {dirIdx} no metadata (resource file?), skip"); continue; }
    Console.Error.WriteLine($"[info] dir {dirIdx} unityVersion={uver} assets={afile.file.AssetInfos.Count}");
    try { am.LoadClassDatabaseFromPackage(uver); }
    catch (Exception ex) { Console.Error.WriteLine($"[warn] classdb load: {ex.Message}"); }

    List<AssetFileInfo> meshInfos;
    try { meshInfos = afile.file.GetAssetsOfType(AssetClassID.Mesh).ToList(); }
    catch (Exception ex) { Console.Error.WriteLine($"[warn] getassets dir{dirIdx}: {ex.Message}"); continue; }
    Console.Error.WriteLine($"[info] dir {dirIdx} mesh assets={meshInfos.Count}");
    if (meshInfos.Count == 0)
    {
        var typeCounts = afile.file.AssetInfos.GroupBy(a => a.TypeId)
            .OrderByDescending(g => g.Count()).Take(15)
            .Select(g => $"{g.Key}:{g.Count()}");
        Console.Error.WriteLine($"[info] dir {dirIdx} top typeIds: {string.Join(" ", typeCounts)}");
    }

    foreach (AssetFileInfo info in meshInfos)
    {
        AssetTypeValueField bf;
        try { bf = am.GetBaseField(afile, info); }
        catch (Exception ex) { Console.Error.WriteLine($"[warn] getbasefield: {ex.Message}"); continue; }
        if (bf == null) continue;

        AssetTypeValueField nameField = bf["m_Name"];
        string name = (nameField != null && !nameField.IsDummy) ? nameField.AsString : "<noname>";
        if (filter != null && !name.Contains(filter, StringComparison.OrdinalIgnoreCase))
            continue;

        int bindCount = ArrayCount(bf, "m_BindPose");
        if (bindCount == 0)
            continue; // only skinned meshes

        int boneHashCount = ArrayCount(bf, "m_BoneNameHashes");

        // Bone name hashes (CRC32 of bone path) — order matches bindpose order.
        var boneHashes = new List<long>();
        AssetTypeValueField bnh = bf["m_BoneNameHashes"];
        if (bnh != null && !bnh.IsDummy && bnh.Children is { Count: > 0 } && bnh.Children[0].Children != null)
        {
            foreach (AssetTypeValueField e in bnh.Children[0].Children)
                boneHashes.Add(e.AsUInt);
        }
        int subMeshCount = ArrayCount(bf, "m_SubMeshes");

        meshHits++;
        List<string>? boneNames = null;
        if (smrMode)
        {
            try { boneNames = ResolveBoneNames(am, afile, info.PathId); }
            catch (Exception ex) { Console.Error.WriteLine($"[warn] bone resolve: {ex.Message}"); }
        }
        results.Add(new
        {
            mesh = name,
            bindposeCount = bindCount,
            boneNameHashCount = boneHashCount,
            subMeshCount = subMeshCount,
            boneNameHashes = boneHashes,
            boneNames,
        });
        Console.Error.WriteLine($"[hit] mesh='{name}' bindposes={bindCount} boneHashes={boneHashCount} boneNames={boneNames?.Count ?? -1}");

        if (filter != null && meshHits >= 1)
            break; // got our target
    }

    if (filter != null && meshHits >= 1)
        break;
}

string json = JsonSerializer.Serialize(new { bundle = bundlePath, filter, meshHits, results },
    new JsonSerializerOptions { WriteIndented = true });
Console.WriteLine(json);
if (outJson != null)
{
    File.WriteAllText(outJson, json, new UTF8Encoding(false));
    Console.Error.WriteLine($"[info] wrote {outJson}");
}

return meshHits > 0 ? 0 : 2;

static int ArrayCount(AssetTypeValueField parent, string fieldName)
{
    AssetTypeValueField f = parent[fieldName];
    if (f == null || f.IsDummy) return 0;
    // Unity arrays are stored as a vector whose first child is the Array container.
    if (f.Children is { Count: > 0 })
    {
        AssetTypeValueField inner = f.Children[0];
        if (inner != null && inner.Children != null && inner.FieldName == "Array")
            return inner.Children.Count;
        return f.Children.Count;
    }
    return 0;
}

// Find a SkinnedMeshRenderer referencing the given mesh PathId, then resolve its
// m_Bones (PPtr<Transform>) → owning GameObject m_Name to recover the bone hierarchy names.
static List<string> ResolveBoneNames(AssetsManager am, AssetsFileInstance afile, long meshPathId)
{
    var names = new List<string>();
    foreach (AssetFileInfo smrInfo in afile.file.GetAssetsOfType(AssetClassID.SkinnedMeshRenderer))
    {
        AssetTypeValueField smr;
        try { smr = am.GetBaseField(afile, smrInfo); } catch { continue; }
        if (smr == null) continue;
        AssetTypeValueField meshPtr = smr["m_Mesh"];
        if (meshPtr == null || meshPtr.IsDummy) continue;
        long pid = meshPtr["m_PathID"].AsLong;
        if (pid != meshPathId) continue;

        AssetTypeValueField bones = smr["m_Bones"];
        if (bones == null || bones.IsDummy || bones.Children is not { Count: > 0 }) continue;
        AssetTypeValueField arr = bones.Children[0];
        if (arr?.Children == null) continue;
        foreach (AssetTypeValueField bonePtr in arr.Children)
        {
            long bonePid = bonePtr["m_PathID"].AsLong;
            string n = ResolveTransformName(am, afile, bonePid);
            names.Add(n);
        }
        break;
    }
    return names;
}

static string ResolveTransformName(AssetsManager am, AssetsFileInstance afile, long transformPathId)
{
    if (transformPathId == 0) return "<null>";
    AssetFileInfo? tInfo = afile.file.GetAssetInfo(transformPathId);
    if (tInfo == null) return $"<missing:{transformPathId}>";
    AssetTypeValueField tf;
    try { tf = am.GetBaseField(afile, tInfo); } catch { return $"<err:{transformPathId}>"; }
    long goPid = tf["m_GameObject"]["m_PathID"].AsLong;
    if (goPid == 0) return $"<nogo:{transformPathId}>";
    AssetFileInfo? goInfo = afile.file.GetAssetInfo(goPid);
    if (goInfo == null) return $"<missinggo:{goPid}>";
    AssetTypeValueField go;
    try { go = am.GetBaseField(afile, goInfo); } catch { return $"<goerr:{goPid}>"; }
    return go["m_Name"].AsString;
}
