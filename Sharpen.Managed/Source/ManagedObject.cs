using Sharpen.Managed.Interop;

using System;
using System.Collections.Generic;
using System.Diagnostics.CodeAnalysis;
using System.Reflection;
using System.Runtime.InteropServices;

namespace Sharpen.Managed;

using static ManagedHost;

internal enum ManagedType
{
    Unknown,

    SByte,
    Byte,
    Short,
    UShort,
    Int,
    UInt,
    Long,
    ULong,

    Float,
    Double,

    Bool,

    String,

    Pointer
}

internal static class ManagedObject
{

    public readonly struct MethodKey : IEquatable<MethodKey>
    {
        public readonly string TypeName;
        public readonly string Name;
        public readonly ManagedType[] Types;
        public readonly int ParameterCount;

        public MethodKey(string InTypeName, string InName, ManagedType[] InTypes, int InParameterCount)
        {
            TypeName = InTypeName;
            Name = InName;
            Types = InTypes;
            ParameterCount = InParameterCount;
        }

        public override bool Equals([NotNullWhen(true)] object? obj) => obj is MethodKey other && Equals(other);

        bool IEquatable<MethodKey>.Equals(MethodKey other)
        {
            if (TypeName != other.TypeName || Name != other.Name)
                return false;

            for (int i = 0; i < Types.Length; i++)
            {
                if (Types[i] != other.Types[i])
                    return false;
            }

            return ParameterCount == other.ParameterCount;
        }

        public override int GetHashCode()
        {
            // NOTE(Peter): Josh Bloch's Hash (from https://stackoverflow.com/questions/263400/what-is-the-best-algorithm-for-overriding-gethashcode)
            unchecked
            {
                int hash = 17;

                hash = hash * 23 + TypeName.GetHashCode();
                hash = hash * 23 + Name.GetHashCode();
                foreach (var type in Types)
                    hash = hash * 23 + type.GetHashCode();
                hash = hash * 23 + ParameterCount.GetHashCode();

                return hash;
            }
        }
    }

    internal static Dictionary<MethodKey, MethodInfo> s_CachedMethods = new Dictionary<MethodKey, MethodInfo>();

    [UnmanagedCallersOnly]
    internal static unsafe IntPtr CreateObject(int InTypeID, Bool32 InWeakRef, IntPtr InParameters, ManagedType* InParameterTypes, int InParameterCount)
    {
        try
        {
            if (!TypeInterface.s_CachedTypes.TryGetValue(InTypeID, out var type))
            {
                LogMessage($"Failed to find type with id '{InTypeID}'.", MessageLevel.Error);
                return IntPtr.Zero;
            }

            ConstructorInfo? constructor = null;

            var currentType = type;
            while (currentType != null)
            {
                ReadOnlySpan<ConstructorInfo> constructors = currentType.GetConstructors(BindingFlags.NonPublic | BindingFlags.Public | BindingFlags.Instance);

                constructor = TypeInterface.FindSuitableMethod(".ctor", InParameterTypes, InParameterCount, constructors);

                if (constructor != null)
                    break;

                currentType = currentType.BaseType;
            }

            if (constructor == null)
            {
                LogMessage($"Failed to find constructor for type {type.FullName} with {InParameterCount} parameters.", MessageLevel.Error);
                return IntPtr.Zero;
            }

            var parameters = Marshalling.MarshalParameterArray(InParameters, InParameterCount, constructor);

            object? result = null;

            if (currentType != type || parameters == null)
            {
                result = TypeInterface.CreateInstance(type);

                if (currentType != type)
                    constructor.Invoke(result, parameters);
            }
            else
            {
                result = TypeInterface.CreateInstance(type, parameters);
            }

            if (result == null)
            {
                LogMessage($"Failed to instantiate type {type.FullName}.", MessageLevel.Error);
            }

            var handle = GCHandle.Alloc(result, InWeakRef ? GCHandleType.Weak : GCHandleType.Normal);
            // TODO: Look into these handles
            //AssemblyLoader.RegisterHandle(type.Assembly, handle);
            return GCHandle.ToIntPtr(handle);
        }
        catch (Exception ex)
        {
            HandleException(ex);
            return IntPtr.Zero;
        }
    }

    [UnmanagedCallersOnly]
    internal static void DestroyObject(IntPtr InObjectHandle)
    {
        try
        {
            GCHandle.FromIntPtr(InObjectHandle).Free();
        }
        catch (Exception ex)
        {
            HandleException(ex);
        }
    }

    private static unsafe MethodInfo? TryGetMethodInfo(Type InType, string InMethodName, ManagedType* InParameterTypes, int InParameterCount, BindingFlags InBindingFlags)
    {
        MethodInfo? methodInfo = null;

        var parameterTypes = new ManagedType[InParameterCount];

        unsafe
        {
            fixed (ManagedType* parameterTypesPtr = parameterTypes)
            {
                ulong size = sizeof(ManagedType) * (ulong)InParameterCount;
                Buffer.MemoryCopy(InParameterTypes, parameterTypesPtr, size, size);
            }
        }

        var methodKey = new MethodKey(InType.FullName, InMethodName, parameterTypes, InParameterCount);

        if (!s_CachedMethods.TryGetValue(methodKey, out methodInfo))
        {
            List<MethodInfo> methods = new(InType.GetMethods(InBindingFlags));

            Type? baseType = InType.BaseType;
            // TODO: Why is this a loop and not just an if
            while (baseType != null)
            {
                methods.AddRange(baseType.GetMethods(InBindingFlags));
                baseType = baseType.BaseType;
            }

            methodInfo = TypeInterface.FindSuitableMethod<MethodInfo>(InMethodName, InParameterTypes, InParameterCount, CollectionsMarshal.AsSpan(methods));

            if (methodInfo == null)
            {
                LogMessage($"Failed to find method '{InMethodName}' for type {InType.FullName} with {InParameterCount} parameters.", MessageLevel.Error);
                return null;
            }

            s_CachedMethods.Add(methodKey, methodInfo);
        }

        return methodInfo;
    }


    [UnmanagedCallersOnly]
    internal static unsafe void InvokeStaticMethod(int InType, NativeString InMethodName, IntPtr InParameters, ManagedType* InParameterTypes, int InParameterCount)
    {
        try
        {
            if (!TypeInterface.s_CachedTypes.TryGetValue(InType, out var type))
            {
                LogMessage($"Cannot invoke method {InMethodName} on a null type.", MessageLevel.Error);
                return;
            }

            var methodInfo = TryGetMethodInfo(type, InMethodName, InParameterTypes, InParameterCount, BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Static);
            var parameters = Marshalling.MarshalParameterArray(InParameters, InParameterCount, methodInfo);

            methodInfo.Invoke(null, parameters);
        }
        catch (Exception ex)
        {
            HandleException(ex);
        }
    }

    [UnmanagedCallersOnly]
    internal static unsafe void InvokeStaticMethodRet(int InType, NativeString InMethodName, IntPtr InParameters, ManagedType* InParameterTypes, int InParameterCount, IntPtr InResultStorage)
    {
        try
        {
            if (!TypeInterface.s_CachedTypes.TryGetValue(InType, out var type))
            {
                LogMessage($"Cannot invoke method {InMethodName} on a null type.", MessageLevel.Error);
                return;
            }

            var methodInfo = TryGetMethodInfo(type, InMethodName, InParameterTypes, InParameterCount, BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Static);
            var methodParameters = Marshalling.MarshalParameterArray(InParameters, InParameterCount, methodInfo);

            object? value = methodInfo.Invoke(null, methodParameters);

            if (value == null)
                return;

            Marshalling.MarshalReturnValue(null, value, methodInfo, InResultStorage);
        }
        catch (Exception ex)
        {
            HandleException(ex);
        }
    }

    [UnmanagedCallersOnly]
    internal static unsafe void InvokeMethod(IntPtr InObjectHandle, NativeString InMethodName, IntPtr InParameters, ManagedType* InParameterTypes, int InParameterCount)
    {
        try
        {
            var target = GCHandle.FromIntPtr(InObjectHandle).Target;

            if (target == null)
            {
                LogMessage($"Cannot invoke method {InMethodName} on a null type.", MessageLevel.Error);
                return;
            }

            var targetType = target.GetType();

            var methodInfo = TryGetMethodInfo(targetType, InMethodName, InParameterTypes, InParameterCount, BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Instance);
            var parameters = Marshalling.MarshalParameterArray(InParameters, InParameterCount, methodInfo);

            methodInfo.Invoke(target, parameters);
        }
        catch (Exception ex)
        {
            HandleException(ex);
        }
    }

    [UnmanagedCallersOnly]
    internal static unsafe void InvokeMethodRet(IntPtr InObjectHandle, NativeString InMethodName, IntPtr InParameters, ManagedType* InParameterTypes, int InParameterCount, IntPtr InResultStorage)
    {
        try
        {
            var target = GCHandle.FromIntPtr(InObjectHandle).Target;

            if (target == null)
            {
                LogMessage($"Cannot invoke method {InMethodName} on object with handle {InObjectHandle}. Target was null.", MessageLevel.Error);
                return;
            }

            var targetType = target.GetType();

            var methodInfo = TryGetMethodInfo(targetType, InMethodName, InParameterTypes, InParameterCount, BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Instance);
            var methodParameters = Marshalling.MarshalParameterArray(InParameters, InParameterCount, methodInfo);

            object? value = methodInfo.Invoke(target, methodParameters);

            if (value == null)
                return;

            Marshalling.MarshalReturnValue(target, value, methodInfo, InResultStorage);
        }
        catch (Exception ex)
        {
            HandleException(ex);
        }
    }

}