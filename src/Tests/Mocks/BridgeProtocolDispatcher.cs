#nullable enable
using System;
using System.Collections.Generic;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using DINOForge.Bridge.Protocol;
using Newtonsoft.Json.Linq;

namespace DINOForge.Tests.Mocks;

/// <summary>
/// Routes JSON-RPC 2.0 method calls to IGameBridge implementation methods.
/// Handles parameter extraction, serialization, and error responses.
/// </summary>
public sealed class BridgeProtocolDispatcher
{
    private readonly IGameBridge _bridge;
    private static readonly Dictionary<string, MethodInfo> MethodCache = new(StringComparer.OrdinalIgnoreCase);

    /// <summary>
    /// Creates a new dispatcher for the given IGameBridge implementation.
    /// </summary>
    /// <param name="bridge">The bridge implementation to dispatch to.</param>
    public BridgeProtocolDispatcher(IGameBridge bridge)
    {
        _bridge = bridge ?? throw new ArgumentNullException(nameof(bridge));
        if (MethodCache.Count == 0)
            CacheMethodNames();
    }

    /// <summary>
    /// Dispatches a JSON-RPC request to the bridge and returns the response.
    /// </summary>
    /// <param name="request">The incoming JSON-RPC request.</param>
    /// <returns>A JSON-RPC response with result or error.</returns>
    public async Task<JsonRpcResponse> DispatchAsync(JsonRpcRequest request)
    {
        try
        {
            if (string.IsNullOrWhiteSpace(request.Method))
            {
                return new JsonRpcResponse
                {
                    Id = request.Id,
                    Error = new JsonRpcError { Code = -32600, Message = "Invalid method name" }
                };
            }

            // Convert camelCase JSON method name to PascalCase C# method name
            string methodName = ToPascalCase(request.Method);
            if (!MethodCache.TryGetValue(methodName, out var methodInfo))
            {
                return new JsonRpcResponse
                {
                    Id = request.Id,
                    Error = new JsonRpcError { Code = -32601, Message = $"Method not found: {request.Method}" }
                };
            }

            // Extract parameters from Params object
            var paramValues = ExtractParameters(methodInfo, request.Params);

            // Invoke the method on the bridge
            object? result = methodInfo.Invoke(_bridge, paramValues);

            // Serialize result to JToken
            JToken resultToken = result is null ? JValue.CreateNull() : JToken.FromObject(result);

            return new JsonRpcResponse
            {
                Id = request.Id,
                Result = resultToken
            };
        }
        catch (TargetInvocationException tie)
        {
            return new JsonRpcResponse
            {
                Id = request.Id,
                Error = new JsonRpcError
                {
                    Code = -32603,
                    Message = "Internal server error",
                    Data = JToken.FromObject(new { Exception = tie.InnerException?.Message ?? "Unknown error" })
                }
            };
        }
        catch (Exception ex)
        {
            return new JsonRpcResponse
            {
                Id = request.Id,
                Error = new JsonRpcError
                {
                    Code = -32603,
                    Message = "Internal server error",
                    Data = JToken.FromObject(new { Exception = ex.Message })
                }
            };
        }
    }

    private static void CacheMethodNames()
    {
        var methods = typeof(IGameBridge).GetMethods(BindingFlags.Public | BindingFlags.Instance);
        foreach (var method in methods)
        {
            MethodCache[method.Name] = method;
        }
    }

    private object?[] ExtractParameters(MethodInfo method, JObject? paramsObj)
    {
        var parameters = method.GetParameters();
        if (parameters.Length == 0)
            return Array.Empty<object?>();

        object?[] values = new object?[parameters.Length];
        paramsObj ??= new JObject();

        for (int i = 0; i < parameters.Length; i++)
        {
            var param = parameters[i];
            string paramName = param.Name ?? "";

            // Try to get the value from params object (case-insensitive key)
            var key = paramsObj.Properties()
                .FirstOrDefault(p => string.Equals(p.Name, paramName, StringComparison.OrdinalIgnoreCase))?
                .Name;

            if (key != null && paramsObj[key] != null)
            {
                values[i] = paramsObj[key]?.ToObject(param.ParameterType);
            }
            else if (param.HasDefaultValue)
            {
                values[i] = param.DefaultValue;
            }
            else
            {
                values[i] = GetDefaultValue(param.ParameterType);
            }
        }

        return values;
    }

    private static object? GetDefaultValue(Type type)
    {
        if (type.IsValueType)
            return Activator.CreateInstance(type);
        return null;
    }

    private static string ToPascalCase(string camelCase)
    {
        if (string.IsNullOrEmpty(camelCase))
            return camelCase;
        return char.ToUpperInvariant(camelCase[0]) + camelCase.Substring(1);
    }
}
