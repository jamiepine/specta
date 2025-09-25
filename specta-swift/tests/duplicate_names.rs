use specta::{Type, TypeCollection};
use specta_swift::{DuplicateNameStrategy, Swift};

// Simulate the Spacedrive scenario with two different LibraryInfo structs
mod libraries {
    use super::*;

    #[derive(Type)]
    pub struct LibraryInfo {
        pub id: u32,
        pub name: String,
        pub path: String,
        pub stats: u32,
    }
}

mod core {
    pub mod status {
        use super::super::*;

        #[derive(Type)]
        pub struct LibraryInfo {
            pub id: u32,
            pub name: String,
            pub is_active: bool,
            pub location_count: u32,
            pub total_entries: u32,
            pub last_sync: String,
        }
    }
}

// Additional test structs with the same name
mod api {
    use super::*;

    #[derive(Type)]
    pub struct UserInfo {
        pub id: u32,
        pub username: String,
        pub email: String,
    }
}

mod database {
    use super::*;

    #[derive(Type)]
    pub struct UserInfo {
        pub id: u32,
        pub name: String,
        pub created_at: String,
        pub is_verified: bool,
    }
}

#[test]
fn test_duplicate_names_warn_strategy() {
    let types = TypeCollection::default()
        .register::<libraries::LibraryInfo>()
        .register::<core::status::LibraryInfo>()
        .register::<api::UserInfo>()
        .register::<database::UserInfo>();

    let swift = Swift::new().duplicate_name_strategy(DuplicateNameStrategy::Warn);

    // Capture stderr to check for warnings
    let output = swift.export(&types).unwrap();

    println!("Generated Swift code with warnings:\n{}", output);

    // Should only have one LibraryInfo definition (the last one)
    let library_info_count = output.matches("public struct LibraryInfo: Codable").count();
    assert_eq!(
        library_info_count, 1,
        "Should have only one LibraryInfo definition"
    );

    // Should only have one UserInfo definition (the last one)
    let user_info_count = output.matches("public struct UserInfo: Codable").count();
    assert_eq!(
        user_info_count, 1,
        "Should have only one UserInfo definition"
    );

    // The last LibraryInfo should have the core::status fields
    assert!(output.contains("let isActive: Bool"));
    assert!(output.contains("let locationCount: UInt32"));
    assert!(output.contains("let totalEntries: UInt32"));
    assert!(output.contains("let lastSync: String"));

    // Should NOT have the libraries::LibraryInfo fields
    assert!(!output.contains("let path: String"));
    assert!(!output.contains("let stats: UInt32"));
}

#[test]
fn test_duplicate_names_error_strategy() {
    let types = TypeCollection::default()
        .register::<libraries::LibraryInfo>()
        .register::<core::status::LibraryInfo>();

    let swift = Swift::new().duplicate_name_strategy(DuplicateNameStrategy::Error);

    // Should fail with an error
    let result = swift.export(&types);
    assert!(result.is_err(), "Should fail with duplicate names error");

    if let Err(e) = result {
        let error_msg = format!("{}", e);
        assert!(
            error_msg.contains("Duplicate type names found"),
            "Error should mention duplicate names"
        );
        assert!(
            error_msg.contains("LibraryInfo"),
            "Error should mention LibraryInfo"
        );
    }
}

#[test]
fn test_duplicate_names_qualify_strategy() {
    let types = TypeCollection::default()
        .register::<libraries::LibraryInfo>()
        .register::<core::status::LibraryInfo>()
        .register::<api::UserInfo>()
        .register::<database::UserInfo>();

    let swift = Swift::new().duplicate_name_strategy(DuplicateNameStrategy::Qualify);

    let output = swift.export(&types).unwrap();

    println!("Generated Swift code with qualified names:\n{}", output);

    // Should have qualified names for both LibraryInfo structs
    assert!(output.contains("public struct LibrariesLibraryInfo: Codable"));
    assert!(output.contains("public struct CoreStatusLibraryInfo: Codable"));

    // Should have qualified names for both UserInfo structs
    assert!(output.contains("public struct ApiUserInfo: Codable"));
    assert!(output.contains("public struct DatabaseUserInfo: Codable"));

    // Each should have their respective fields
    // LibrariesLibraryInfo should have path and stats
    let libraries_section = output.find("struct LibrariesLibraryInfo").unwrap();
    let libraries_end = output[libraries_section..].find("}").unwrap() + libraries_section;
    let libraries_content = &output[libraries_section..libraries_end];
    assert!(libraries_content.contains("let path: String"));
    assert!(libraries_content.contains("let stats: UInt32"));

    // CoreStatusLibraryInfo should have isActive, locationCount, etc.
    let core_section = output.find("struct CoreStatusLibraryInfo").unwrap();
    let core_end = output[core_section..].find("}").unwrap() + core_section;
    let core_content = &output[core_section..core_end];
    assert!(core_content.contains("let isActive: Bool"));
    assert!(core_content.contains("let locationCount: UInt32"));
}

#[test]
fn test_duplicate_names_custom_strategy() {
    let types = TypeCollection::default()
        .register::<libraries::LibraryInfo>()
        .register::<core::status::LibraryInfo>();

    let swift = Swift::new().duplicate_name_strategy(DuplicateNameStrategy::Custom(|ndt| {
        // Custom naming: add module path as prefix
        let module_path = ndt.module_path();
        if module_path.contains("libraries") {
            "LibrariesListLibraryInfo".to_string()
        } else if module_path.contains("core::status") {
            "CoreStatusLibraryInfo".to_string()
        } else {
            ndt.name().to_string()
        }
    }));

    let output = swift.export(&types).unwrap();

    println!("Generated Swift code with custom naming:\n{}", output);

    // Should have custom names
    assert!(output.contains("public struct LibrariesListLibraryInfo: Codable"));
    assert!(output.contains("public struct CoreStatusLibraryInfo: Codable"));

    // Each should have their respective fields
    let libraries_section = output.find("struct LibrariesListLibraryInfo").unwrap();
    let libraries_end = output[libraries_section..].find("}").unwrap() + libraries_section;
    let libraries_content = &output[libraries_section..libraries_end];
    assert!(libraries_content.contains("let path: String"));
    assert!(libraries_content.contains("let stats: UInt32"));

    let core_section = output.find("struct CoreStatusLibraryInfo").unwrap();
    let core_end = output[core_section..].find("}").unwrap() + core_section;
    let core_content = &output[core_section..core_end];
    assert!(core_content.contains("let isActive: Bool"));
    assert!(core_content.contains("let locationCount: UInt32"));
}

#[test]
fn test_no_duplicates_default_behavior() {
    // Test that non-duplicate names work normally
    #[derive(Type)]
    struct UniqueStruct {
        pub id: u32,
        pub name: String,
    }

    let types = TypeCollection::default().register::<UniqueStruct>();

    let swift = Swift::default(); // Uses Warn strategy by default
    let output = swift.export(&types).unwrap();

    println!("Generated Swift code for unique struct:\n{}", output);

    // Should work normally without any warnings
    assert!(output.contains("public struct UniqueStruct: Codable"));
    assert!(output.contains("let id: UInt32"));
    assert!(output.contains("let name: String"));
}

#[test]
fn test_duplicate_names_with_enums() {
    mod first {
        use super::*;

        #[derive(Type)]
        pub enum Status {
            Active,
            Inactive,
        }
    }

    mod second {
        use super::*;

        #[derive(Type)]
        pub enum Status {
            Online,
            Offline,
            Away,
        }
    }

    let types = TypeCollection::default()
        .register::<first::Status>()
        .register::<second::Status>();

    let swift = Swift::new().duplicate_name_strategy(DuplicateNameStrategy::Qualify);

    let output = swift.export(&types).unwrap();

    println!("Generated Swift code for duplicate enums:\n{}", output);

    // Should have qualified enum names
    assert!(output.contains("public enum FirstStatus: Codable"));
    assert!(output.contains("public enum SecondStatus: Codable"));

    // Each should have their respective variants
    assert!(output.contains("case active"));
    assert!(output.contains("case inactive"));
    assert!(output.contains("case online"));
    assert!(output.contains("case offline"));
    assert!(output.contains("case away"));
}
