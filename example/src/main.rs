use sharpen::{
    TypeCacheError, TypeFns,
    assembly::{AssemblyLoadError, ManagedAssembly},
    host_instance::{CoralInitError, HostInstance, HostSettings},
    managed_object::ManagedObjectFns,
    message_level::MessageLevel,
    meta_info::Attribute,
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

#[allow(unused)]
struct MyVec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

// TODO: Make a function to get a fn pointer instead of searching by name? Do some sort of type/safety checks on the C# side
// TODO: Maybe test with F# or other CLR language

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

    let example_type = assembly
        .get_type("Example.Managed.ExampleClass")
        .map_err(|err| ExampleError::TypeCacheError(err))?;

    // TODO: Safety of specifying wrong return type or argument type?
    let value = example_type.invoke_static_method::<f32>("StaticMethod", (50.0f32,));
    println!("Value in rust: {value}");

    let custom_attribute_type = assembly
        .get_type("Example.Managed.CustomAttribute")
        .map_err(|err| ExampleError::TypeCacheError(err))?;

    for attribute in &example_type.get_attributes() {
        // TODO: Mutability
        if *unsafe {
            (&*attribute as *const _ as *mut Attribute)
                .as_mut()
                .unwrap()
        }
        .get_type()
            == *custom_attribute_type
        {
            println!(
                "CustomAttribute: {}",
                attribute.get_field_value::<_, f32>("Value")
            );
        }
    }

    let example_instance = example_type.create_instance((50i32,));
    example_instance.invoke_method::<()>(
        "Void MemberMethod(MyVec3)",
        (MyVec3 {
            x: 10.0,
            y: 10.0,
            z: 10.0,
        },),
    );

    example_instance.set_property_value("PublicProp", 10i32);
    // TODO: Remove the need for _, in generic
    println!(
        "PublicProp: {}",
        example_instance.get_property_value::<_, i32>("PublicProp")
    );

    example_instance.set_field_value("myPrivateValue", 10i32);
    println!(
        "myPrivateValue: {}",
        example_instance.get_field_value::<_, i32>("myPrivateValue")
    );

    // TODO: Arrays and maybe rename CSharpNativeString

    example_instance.destroy();

    Ok(())
}
