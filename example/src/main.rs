use sharpen::{
    assembly::AssemblyLoadError,
    csharp_type::TypeFns,
    host_instance::{HostInstance, HostSettings, SharpenInitError},
    managed_object::ManagedObjectFns,
    message_level::MessageLevel,
    type_cache::TypeCacheError,
};

#[derive(Debug, Clone, Copy)]
#[allow(unused)]
enum ExampleError {
    SharpenInitError(SharpenInitError),
    AssemblyLoadError(AssemblyLoadError),
    TypeCacheError(TypeCacheError),
}

// TODO: Make a function to get a fn pointer instead of searching by name? Do some sort of type/safety checks on the C# side
// TODO: Maybe test with F# or other CLR language

fn main() -> Result<(), ExampleError> {
    let exception_callback = |message| {
        println!("[Sharpen](Error): {message}");
    };

    let host_instance = HostInstance::initialize(HostSettings {
        sharpen_directory: std::path::PathBuf::from("./Sharpen.Managed.Output"),
        message_callback: None,
        messsage_filter: MessageLevel::Info,
        exception_callback: Some(exception_callback),
    })
    .map_err(|err| ExampleError::SharpenInitError(err))?;

    let mut assembly_load_context = host_instance.create_assembly_load_context("ExampleContext");

    let assembly_path =
        std::path::PathBuf::from("./Example.Managed/bin/Debug/net8.0/Example.Managed.dll");
    let assembly = assembly_load_context
        .load_assembly(&assembly_path)
        .map_err(|err| ExampleError::AssemblyLoadError(err))?;

    let example_type = assembly
        .get_type("Example.Managed.ExampleClass")
        .map_err(|err| ExampleError::TypeCacheError(err))?;

    // TODO: Safety of specifying wrong return type or argument type?
    let value = example_type.invoke_static_method::<f32>("StaticMethod", (50.0f32,));
    println!("Value in rust: {value}");

    let example_instance = example_type.create_instance((20i32,));
    example_instance.invoke_method::<()>("MemberMethod", ());

    example_instance.destroy();

    Ok(())
}
