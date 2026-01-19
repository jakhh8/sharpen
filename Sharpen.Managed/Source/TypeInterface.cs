using System;
using System.Collections.Generic;
using System.Reflection;
using System.Runtime.InteropServices;
using Sharpen.Managed.Interop;

namespace Sharpen.Managed;

using static ManagedHost;

internal static class TypeInterface
{

	internal readonly static UniqueIdList<Type> s_CachedTypes = new();

	internal static Type? FindType(string? InTypeName)
	{
		var type = Type.GetType(InTypeName!);
		// TODO: Potentially add Resolving here
		/* var type = Type.GetType(InTypeName!,
            (name) => AssemblyLoader.ResolveAssembly(null, name),
            (assembly, name, ignore) =>
            {
                return assembly != null ? assembly.GetType(name, false, ignore) : Type.GetType(name, false, ignore);
            }
        ); */

		return type;
	}

	internal static object? CreateInstance(Type InType, params object?[]? InArguments)
	{
		return InType.Assembly.CreateInstance(InType.FullName ?? string.Empty, false, BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Instance, null, InArguments!, null, null);
	}

	private static Dictionary<Type, ManagedType> s_TypeConverters = new()
	{
		{ typeof(sbyte), ManagedType.SByte },
		{ typeof(byte), ManagedType.Byte },
		{ typeof(short), ManagedType.Short },
		{ typeof(ushort), ManagedType.UShort },
		{ typeof(int), ManagedType.Int },
		{ typeof(uint), ManagedType.UInt },
		{ typeof(long), ManagedType.Long },
		{ typeof(ulong), ManagedType.ULong },
		{ typeof(float), ManagedType.Float },
		{ typeof(double), ManagedType.Double },
		{ typeof(Bool32), ManagedType.Bool },
		{ typeof(bool), ManagedType.Bool },
		{ typeof(NativeString), ManagedType.String },
		{ typeof(string), ManagedType.String },
	};

	internal static unsafe T? FindSuitableMethod<T>(string? InMethodName, ManagedType* InParameterTypes, int InParameterCount, ReadOnlySpan<T> InMethods) where T : MethodBase
	{
		if (InMethodName == null)
			return null;

		T? result = null;

		foreach (var methodInfo in InMethods)
		{
			var methodParams = methodInfo.GetParameters();

			if (methodParams.Length != InParameterCount)
				continue;

			// Check if the method name matches the signature of methodInfo, if so we ignore the automatic type checking
			if (InMethodName == methodInfo.ToString())
			{
				result = methodInfo;
				break;
			}

			if (methodInfo.Name != InMethodName)
				continue;

			int matchingTypes = 0;

			for (int i = 0; i < methodParams.Length; i++)
			{
				ManagedType paramType;

				if (methodParams[i].ParameterType.IsPointer || methodParams[i].ParameterType == typeof(IntPtr))
				{
					paramType = ManagedType.Pointer;
				}
				else if (!s_TypeConverters.TryGetValue(methodParams[i].ParameterType, out paramType))
				{
					paramType = ManagedType.Unknown;
				}

				if (paramType == InParameterTypes[i])
				{
					matchingTypes++;
				}
			}

			if (matchingTypes == InParameterCount)
			{
				result = methodInfo;
				break;
			}
		}

		return result;
	}


	[UnmanagedCallersOnly]
	internal static unsafe void GetAssemblyTypes(int InContextId, int InAssemblyId, int* OutTypes, int* OutTypeCount)
	{
		try
		{
			if (!AssemblyLoader.TryGetAssembly(InContextId, InAssemblyId, out var assembly))
			{
				LogMessage($"Couldn't get types for assembly '{InAssemblyId}', assembly not found.", MessageLevel.Error);
				return;
			}

			if (assembly == null)
			{
				LogMessage($"Couldn't get types for assembly '{InAssemblyId}', assembly was null.", MessageLevel.Error);
				return;
			}

			ReadOnlySpan<Type> assemblyTypes = assembly.GetTypes();

			if (OutTypeCount != null)
				*OutTypeCount = assemblyTypes.Length;

			if (OutTypes == null)
				return;

			for (int i = 0; i < assemblyTypes.Length; i++)
			{
				OutTypes[i] = s_CachedTypes.Add(assemblyTypes[i]);
			}
		}
		catch (Exception ex)
		{
			HandleException(ex);
		}
	}

	[UnmanagedCallersOnly]
	internal static unsafe void GetTypeId(NativeString InName, int* OutType)
	{
		try
		{
			var type = FindType(InName);

			if (type == null)
			{
				LogMessage($"Failed to find type with name '{InName}'.", MessageLevel.Error);
				return;
			}

			*OutType = s_CachedTypes.Add(type);
		}
		catch (Exception e)
		{
			HandleException(e);
		}
	}

	[UnmanagedCallersOnly]
	internal static unsafe NativeString GetFullTypeName(int InType)
	{
		try
		{
			if (!s_CachedTypes.TryGetValue(InType, out var type))
				return NativeString.Null();

			return type.FullName;
		}
		catch (Exception e)
		{
			HandleException(e);
			return NativeString.Null();
		}
	}

	[UnmanagedCallersOnly]
	internal static unsafe NativeString GetAssemblyQualifiedName(int InType)
	{
		try
		{
			if (!s_CachedTypes.TryGetValue(InType, out var type))
				return NativeString.Null();

			return type.AssemblyQualifiedName;
		}
		catch (Exception e)
		{
			HandleException(e);
			return NativeString.Null();
		}
	}

	[UnmanagedCallersOnly]
	internal static unsafe void GetBaseType(int InType, int* OutBaseType)
	{
		try
		{
			if (!s_CachedTypes.TryGetValue(InType, out var type) || OutBaseType == null)
				return;

			if (type.BaseType == null)
			{
				*OutBaseType = 0;
				return;
			}

			*OutBaseType = s_CachedTypes.Add(type.BaseType);
		}
		catch (Exception e)
		{
			HandleException(e);
		}
	}

	[UnmanagedCallersOnly]
	internal static int GetTypeSize(int InType)
	{
		try
		{
			if (!s_CachedTypes.TryGetValue(InType, out var type))
				return -1;

			return Marshal.SizeOf(type);
		}
		catch (Exception e)
		{
			HandleException(e);
			return -1;
		}
	}

}