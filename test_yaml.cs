using System;
using YamlDotNet.Serialization;
using YamlDotNet.Serialization.NamingConventions;

class TestYaml
{
    static void Main()
    {
        Console.WriteLine("Starting YAML test...");
        
        var yaml = @"version: 1.0.0
pack_id: test-pack
target_unity_version: 2021.3.45f2";
        
        Console.WriteLine("Creating deserializer...");
        var deserializer = new DeserializerBuilder()
            .WithNamingConvention(UnderscoredNamingConvention.Instance)
            .IgnoreUnmatchedProperties()
            .Build();
        
        Console.WriteLine("Deserializing minimal YAML...");
        dynamic result = deserializer.Deserialize<dynamic>(yaml);
        Console.WriteLine("Deserialization complete!");
    }
}
