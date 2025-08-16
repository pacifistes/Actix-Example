use serde::Serialize;
use strum::{Display, EnumString};

// Role enumeration
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, EnumString, Display)]
#[serde(rename_all = "PascalCase")]
pub enum Role {
    Admin,
    CarManager,
    MotorbikeManager,
    Customer1,
    Customer2,
}

// Identity structure
#[derive(Clone, Debug, Serialize)]
pub struct Identity {
    pub role: Role,
    pub user_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_role_admin_serialize() {
        // Test serialization (JSON serialization is still needed for API responses)
        let admin_role = Role::Admin;
        let serialized =
            serde_json::to_string(&admin_role).expect("Failed to serialize Role::Admin");

        // The role should serialize to "Admin" due to PascalCase rename
        assert_eq!(serialized, "\"Admin\"");

        // Test other roles too
        assert_eq!(
            serde_json::to_string(&Role::CarManager).unwrap(),
            "\"CarManager\""
        );
        assert_eq!(
            serde_json::to_string(&Role::Customer1).unwrap(),
            "\"Customer1\""
        );
    }

    #[test]
    fn test_strum_role_parsing() {
        // Test Strum's FromStr implementation
        let admin_from_str = Role::from_str("Admin").expect("Failed to parse Admin");
        assert_eq!(admin_from_str, Role::Admin);

        let car_manager_from_str =
            Role::from_str("CarManager").expect("Failed to parse CarManager");
        assert_eq!(car_manager_from_str, Role::CarManager);

        // Test Display trait (default Strum behavior uses variant names as-is)
        assert_eq!(Role::Admin.to_string(), "Admin");
        assert_eq!(Role::CarManager.to_string(), "CarManager");

        // Test invalid role
        let invalid_role = Role::from_str("InvalidRole");
        assert!(invalid_role.is_err());
    }
}
