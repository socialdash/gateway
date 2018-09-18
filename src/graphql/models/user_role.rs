use stq_types::UserId;

#[derive(GraphQLEnum, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[graphql(description = "Users microservice role")]
pub enum UserMicroserviceRole {
    Superuser,
    Moderator,
    User,
}

#[derive(GraphQLEnum, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[graphql(description = "Stores microservice role")]
pub enum StoresMicroserviceRole {
    Superuser,
    Moderator,
    PlatformAdmin,
    User,
}

#[derive(GraphQLInputObject, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[graphql(description = "New role input object")]
pub struct NewUsersRoleInput {
    #[graphql(description = "Client mutation id.")]
    #[serde(skip_serializing)]
    pub client_mutation_id: String,
    #[graphql(description = "User id")]
    pub user_id: i32,
    #[graphql(description = "New stores Role")]
    pub role: UserMicroserviceRole,
}

#[derive(GraphQLInputObject, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[graphql(description = "New role input object")]
pub struct NewStoresRoleInput {
    #[graphql(description = "Client mutation id.")]
    #[serde(skip_serializing)]
    pub client_mutation_id: String,
    #[graphql(description = "User id")]
    pub user_id: i32,
    #[graphql(description = "New stores Role")]
    pub role: StoresMicroserviceRole,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct UsersRoles {
    pub user_id: UserId,
    pub roles: Vec<UserMicroserviceRole>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct StoresRoles {
    pub user_id: UserId,
    pub roles: Vec<StoresMicroserviceRole>,
}
