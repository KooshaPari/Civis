#!/bin/bash
dotnet build src/DINOForge.sln && echo "Build succeeded" || echo "Build FAILED"
