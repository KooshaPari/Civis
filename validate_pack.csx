#!/usr/bin/env dotnet
#r "src/SDK/bin/Release/netstandard2.0/DINOForge.SDK.dll"
#r "System.Collections"

using DINOForge.SDK.Models;
using System.IO;
using YamlDotNet.Serialization;
using YamlDotNet.Serialization.NamingConventions;

var packPath = "packs/scenario-tutorial";
var packYamlPath = Path.Combine(packPath, "pack.yaml");

var deserializer = new DeserializerBuilder()
    .WithNamingConvention(new UnderscoredNamingConvention())
    .Build();

try
{
    var yaml = File.ReadAllText(packYamlPath);
    var pack = deserializer.Deserialize<PackManifest>(yaml);
    
    Console.WriteLine($"✓ Pack loaded: {pack.Id} v{pack.Version}");
    Console.WriteLine($"  Name: {pack.Name}");
    Console.WriteLine($"  Type: {pack.Type}");
}
catch (Exception ex)
{
    Console.WriteLine($"✗ Error: {ex.Message}");
}
