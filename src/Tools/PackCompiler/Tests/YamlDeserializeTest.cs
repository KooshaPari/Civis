using System;
using System.IO;
using System.Text;
using Xunit;
using YamlDotNet.Serialization;
using YamlDotNet.Serialization.NamingConventions;
using DINOForge.Tools.PackCompiler.Models;

namespace DINOForge.Tools.PackCompiler.Tests
{
    public class YamlDeserializeTest
    {
        [Fact]
        public void DeserializeMinimalYaml()
        {
            var yaml = @"version: 1.0.0
pack_id: test-pack
target_unity_version: 2021.3.45f2
asset_settings:
  base_path: assets/raw
  output_path: output
materials: {}
phases:
  v0_7_0_critical:
    description: Critical Phase
    models: []
build:
  output_directory: output
  addressables_output: addressables";

            var deserializer = new DeserializerBuilder()
                .WithNamingConvention(UnderscoredNamingConvention.Instance)
                .IgnoreUnmatchedProperties()
                .Build();

            var result = deserializer.Deserialize<AssetPipelineConfig>(yaml);
            Assert.NotNull(result);
            Assert.Equal("test-pack", result.PackId);
        }

        [Fact]
        public void DeserializeActualFile()
        {
            // Navigate from bin/Release/net9.0/ to repo root (6 levels up)
            string configPath = Path.Combine(AppContext.BaseDirectory, "..", "..", "..", "..", "..", "..", "packs", "warfare-starwars", "asset_pipeline.yaml");
            configPath = Path.GetFullPath(configPath);
            Assert.True(File.Exists(configPath), $"File not found: {configPath}");

            var yaml = File.ReadAllText(configPath, Encoding.UTF8);
            var deserializer = new DeserializerBuilder()
                .WithNamingConvention(UnderscoredNamingConvention.Instance)
                .IgnoreUnmatchedProperties()
                .Build();

            var result = deserializer.Deserialize<AssetPipelineConfig>(yaml);
            Assert.NotNull(result);
            Assert.NotNull(result.PackId);
        }
    }
}
