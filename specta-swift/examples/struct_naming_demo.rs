use specta::{Type, TypeCollection};
use specta_swift::{StructNamingStrategy, Swift};

fn main() {
    #[derive(Type)]
    struct User {
        id: u32,
        name: String,
        email: Option<String>,
    }

    #[derive(Type)]
    enum ApiResponse<T> {
        Success { data: T, status_code: u16 },
        Error { message: String, code: u32 },
        Loading { progress: f32 },
    }

    let types = TypeCollection::default()
        .register::<User>()
        .register::<ApiResponse<()>>();

    println!("🔧 Struct Naming Configuration Demo");
    println!("{}", "=".repeat(60));

    // Default behavior (AutoRename)
    println!("\n📝 Default Behavior (AutoRename):");
    println!("{}", "-".repeat(40));
    let default_swift = Swift::new().export(&types).unwrap();
    println!("{}", default_swift);

    // KeepOriginal behavior
    println!("\n📝 KeepOriginal Behavior:");
    println!("{}", "-".repeat(40));
    let keep_original_swift = Swift::new()
        .struct_naming(StructNamingStrategy::KeepOriginal)
        .export(&types)
        .unwrap();
    println!("{}", keep_original_swift);

    println!("\n✅ Demo completed!");
    println!("\n🔍 Key Differences:");
    println!("• Default (AutoRename): ApiResponse::Success → ApiResponseSuccessData");
    println!("• KeepOriginal: ApiResponse::Success → SuccessData");
}
