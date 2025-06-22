use sharpen::{
    InvokeStaticMethod, TypeCacheError,
    assembly::{AssemblyLoadError, ManagedAssembly},
    host_instance::{CoralInitError, HostInstance, HostSettings},
    message_level::MessageLevel,
};

#[derive(Debug, Clone, Copy)]
#[allow(unused)]
enum ExampleError {
    CoralInitError(CoralInitError),
    AssemblyLoadError(AssemblyLoadError),
    TypeCacheError(TypeCacheError),
}

fn test_internal_call(value: f32) -> f32 {
    println!("Value in Icall: {value}");

    value - 10.0
}

fn main() -> Result<(), ExampleError> {
    let exception_callback = |message| {
        println!("[Sharpen](Error): {message}");
    };

    let host_instance = HostInstance::initialize(HostSettings {
        coral_directory: std::path::PathBuf::from("./Coral.Managed.Output"),
        message_callback: None,
        messsage_filter: MessageLevel::Info,
        exception_callback: Some(exception_callback),
    })
    .map_err(|err| ExampleError::CoralInitError(err))?;

    let mut assembly_load_context = host_instance.create_assembly_load_context("ExampleContext");

    let assembly_path =
        std::path::PathBuf::from("./Example.Managed/bin/Debug/net8.0/Example.Managed.dll");
    let assembly = assembly_load_context
        .load_assembly(&assembly_path)
        .map_err(|err| ExampleError::AssemblyLoadError(err))?;

    unsafe {
        // TODO: Mutability
        (assembly.as_ref() as *const _ as *mut ManagedAssembly)
            .as_mut()
            .unwrap()
            .add_internal_call(
                "Example.Managed.ExampleClass",
                "TestInternalCall",
                test_internal_call as _,
            );
    }
    assembly.upload_internal_calls();
    std::hint::black_box(test_internal_call);

    let example_type = assembly
        .get_type("Example.Managed.ExampleClass")
        .map_err(|err| ExampleError::TypeCacheError(err))?;

    // TODO: Safety of specifying wrong return type or argument type?
    let value = example_type.invoke_static_method::<f32>("StaticMethod", (50.0f32,));
    println!("Value in rust: {value}");

    Ok(())
}
