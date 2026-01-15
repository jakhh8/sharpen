using Sharpen.Managed.Interop;

using System;
using System.Collections.Generic;
using System.IO;
using System.IO.MemoryMappedFiles;
using System.Reflection;
using System.Runtime.InteropServices;
using System.Runtime.Loader;

namespace Sharpen.Managed;

using static ManagedHost;

public enum AssemblyLoadStatus
{
    Success, FileNotFound, FileLoadFailure, InvalidFilePath, InvalidAssembly, UnknownError
}

public static class AssemblyLoader
{

    private static readonly Dictionary<int, AssemblyLoadContext> s_LoadContexts = new();

    private static readonly Dictionary<int, Dictionary<int, Assembly>> s_AssemblyCache = new();

    private static AssemblyLoadStatus s_LastLoadStatus = AssemblyLoadStatus.Success;

    private static readonly Dictionary<Type, AssemblyLoadStatus> s_LoadErrorLookup = new();

    static AssemblyLoader()
    {
        s_LoadErrorLookup.Add(typeof(BadImageFormatException), AssemblyLoadStatus.InvalidAssembly);
        s_LoadErrorLookup.Add(typeof(FileNotFoundException), AssemblyLoadStatus.FileNotFound);
        s_LoadErrorLookup.Add(typeof(FileLoadException), AssemblyLoadStatus.FileLoadFailure);
        s_LoadErrorLookup.Add(typeof(ArgumentNullException), AssemblyLoadStatus.InvalidFilePath);
        s_LoadErrorLookup.Add(typeof(ArgumentException), AssemblyLoadStatus.InvalidFilePath);
    }

    internal static bool TryGetAssembly(int InContextId, int InAssemblyId, out Assembly? OutAssembly)
    {
        return s_AssemblyCache[InContextId].TryGetValue(InAssemblyId, out OutAssembly);
    }

    [UnmanagedCallersOnly]
    internal static int CreateAssemblyLoadContext(NativeString InName)
    {
        string? name = InName;

        if (name == null)
            return -1;

        var alc = new AssemblyLoadContext(name, true);
        // TODO: Resolving
        alc.Unloading += ctx => s_AssemblyCache.Remove(ctx.Name!.GetHashCode());

        // TODO: Exception handling (both alc and alc.Name could be null)
        var contextId = alc.Name!.GetHashCode();
        s_LoadContexts.Add(contextId, alc);
        s_AssemblyCache.Add(contextId, new());

        return contextId;
    }

    [UnmanagedCallersOnly]
    internal static void UnloadAssemblyLoadContext(int InContextId)
    {
        if (!s_LoadContexts.TryGetValue(InContextId, out var alc))
        {
            LogMessage($"Cannot unload AssemblyLoadContext '{InContextId}', it was either never loaded or already unloaded.", MessageLevel.Warning);
            return;
        }

        if (alc == null)
        {
            LogMessage($"AssemblyLoadContext '{InContextId}' was found in dictionary but was null. This is most likely a bug.", MessageLevel.Error);
            return;
        }

        s_LoadContexts.Remove(InContextId);
        alc.Unload();
    }

    [UnmanagedCallersOnly]
    internal static int LoadAssembly(int InContextId, NativeString InAssemblyFilePath)
    {
        try
        {
            if (string.IsNullOrEmpty(InAssemblyFilePath))
            {
                s_LastLoadStatus = AssemblyLoadStatus.InvalidFilePath;
                return -1;
            }

            if (!File.Exists(InAssemblyFilePath))
            {
                LogMessage($"Failed to load assembly '{InAssemblyFilePath}', file not found.", MessageLevel.Error);
                s_LastLoadStatus = AssemblyLoadStatus.FileNotFound;
                return -1;
            }

            if (!s_LoadContexts.TryGetValue(InContextId, out var alc))
            {
                LogMessage($"Failed to load assembly '{InAssemblyFilePath}', couldn't find AssemblyLoadContext with id {InContextId}.", MessageLevel.Error);
                s_LastLoadStatus = AssemblyLoadStatus.UnknownError;
                return -1;
            }

            if (alc == null)
            {
                LogMessage($"Failed to load assembly '{InAssemblyFilePath}', AssemblyLoadContext with id {InContextId} was null.", MessageLevel.Error);
                s_LastLoadStatus = AssemblyLoadStatus.UnknownError;
                return -1;
            }

            Assembly? assembly = null;

            using (var file = MemoryMappedFile.CreateFromFile(InAssemblyFilePath!))
            {
                using var stream = file.CreateViewStream();
                assembly = alc.LoadFromStream(stream);
            }

            LogMessage($"Loading assembly '{InAssemblyFilePath}'", MessageLevel.Info);
            var assemblyName = assembly.GetName();
            int assemblyId = assemblyName.Name!.GetHashCode();
            s_AssemblyCache[InContextId].Add(assemblyId, assembly);
            s_LastLoadStatus = AssemblyLoadStatus.Success;
            return assemblyId;
        }
        catch (Exception ex)
        {
            s_LoadErrorLookup.TryGetValue(ex.GetType(), out s_LastLoadStatus);
            HandleException(ex);
            return -1;
        }
    }

    [UnmanagedCallersOnly]
    internal static unsafe int LoadAssemblyFromMemory(int InContextId, byte* data, long dataLength)
    {
        try
        {
            if (!s_LoadContexts.TryGetValue(InContextId, out var alc))
            {
                LogMessage($"Failed to load assembly, couldn't find AssemblyLoadContext with id {InContextId}.", MessageLevel.Error);
                s_LastLoadStatus = AssemblyLoadStatus.UnknownError;
                return -1;
            }

            if (alc == null)
            {
                LogMessage($"Failed to load assembly, AssemblyLoadContext with id {InContextId} was null.", MessageLevel.Error);
                s_LastLoadStatus = AssemblyLoadStatus.UnknownError;
                return -1;
            }

            Assembly? assembly = null;

            using (var stream = new UnmanagedMemoryStream(data, dataLength))
            {
                assembly = alc.LoadFromStream(stream);
            }

            LogMessage($"Loading assembly '{assembly.FullName}'", MessageLevel.Info);
            var assemblyName = assembly.GetName();
            int assemblyId = assemblyName.Name!.GetHashCode();
            s_AssemblyCache[InContextId].Add(assemblyId, assembly);
            s_LastLoadStatus = AssemblyLoadStatus.Success;
            return assemblyId;
        }
        catch (Exception ex)
        {
            s_LoadErrorLookup.TryGetValue(ex.GetType(), out s_LastLoadStatus);
            HandleException(ex);
            return -1;
        }
    }

    [UnmanagedCallersOnly]
    internal static AssemblyLoadStatus GetLastLoadStatus() => s_LastLoadStatus;

    [UnmanagedCallersOnly]
    internal static NativeString GetAssemblyName(int InContextId, int InAssemblyId)
    {
        if (!s_AssemblyCache[InContextId].TryGetValue(InAssemblyId, out var assembly))
        {
            LogMessage($"Couldn't get assembly name for assembly '{InAssemblyId}', assembly not found in dictionary.", MessageLevel.Error);
            return "";
        }

        // TODO: assemblyName.Name could be null?
        var assemblyName = assembly.GetName();
        return assemblyName.Name;
    }

}