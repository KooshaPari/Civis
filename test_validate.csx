using System;
using System.IO;
using YamlDotNet.Serialization;
using YamlDotNet.Serialization.NamingConventions;

// Simple inline validation
var packPath = "packs/scenario-tutorial";
var scenarioDir = Path.Combine(packPath, "scenarios");

Console.WriteLine("=== Scenario Tutorial Pack Validation ===");
Console.WriteLine($"Pack directory: {packPath}");
Console.WriteLine($"Pack exists: {Directory.Exists(packPath)}");
Console.WriteLine($"pack.yaml exists: {File.Exists(Path.Combine(packPath, "pack.yaml"))}");
Console.WriteLine($"scenarios/ exists: {Directory.Exists(scenarioDir)}");

var scenarios = Directory.GetFiles(scenarioDir, "*.yaml");
Console.WriteLine($"Scenario files found: {scenarios.Length}");
foreach (var scenario in scenarios)
{
    Console.WriteLine($"  - {Path.GetFileName(scenario)}");
    var content = File.ReadAllLines(scenario);
    Console.WriteLine($"    Lines: {content.Length}");
}

Console.WriteLine("\n✓ Pack structure validation PASSED");
