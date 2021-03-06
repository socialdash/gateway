//! EAV model attributes
use juniper::ID as GraphqlID;
use uuid::Uuid;

use stq_static_resources::{AttributeType, Translation, TranslationInput};
use stq_types::{AttributeId, AttributeValueCode, AttributeValueId};

#[derive(Deserialize, Debug, Clone)]
pub struct Attribute {
    pub id: AttributeId,
    pub name: Vec<Translation>,
    pub value_type: AttributeType,
    pub meta_field: Option<AttributeMetaField>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct AttributeMetaField {
    pub values: Option<Vec<String>>,                      //todo deprecated
    pub translated_values: Option<Vec<Vec<Translation>>>, //todo deprecated
    pub ui_element: UIType,
}

#[derive(Deserialize, Debug, Clone)]
pub struct AttributeValue {
    pub id: AttributeValueId,
    pub attr_id: AttributeId,
    pub code: AttributeValueCode,
    pub translations: Option<Vec<Translation>>,
}

#[derive(GraphQLEnum, Deserialize, Serialize, Clone, Debug, PartialEq)]
#[graphql(name = "UIType", description = "UI element type")]
pub enum UIType {
    Combobox,
    Radiobutton,
    Checkbox,
    ColorPicker,
}

#[derive(GraphQLInputObject, Serialize, Debug, Clone, PartialEq)]
#[graphql(description = "Attribute meta field input object")]
pub struct AttributeMetaFieldInput {
    #[graphql(description = "Possible values of attribute")]
    #[graphql(deprecation = "Use parent's \"value\" field")]
    pub values: Option<Vec<String>>,
    #[graphql(description = "Possible values of attribute with translation")]
    #[graphql(deprecation = "Use parent's \"value\" field")]
    pub translated_values: Option<Vec<Vec<TranslationInput>>>,
    #[graphql(description = "UI element type ")]
    pub ui_element: UIType,
}

#[derive(GraphQLInputObject, Serialize, Debug, Clone, PartialEq)]
#[graphql(description = "Update attribute input object")]
pub struct UpdateAttributeInput {
    #[graphql(description = "Client mutation id.")]
    #[serde(skip_serializing)]
    pub client_mutation_id: String,
    #[graphql(description = "Id of a attribute.")]
    #[serde(skip_serializing)]
    pub id: GraphqlID,
    #[graphql(description = "New name of an attribute")]
    pub name: Option<Vec<TranslationInput>>,
    #[graphql(description = "New meta_field of an attribute")]
    pub meta_field: Option<AttributeMetaFieldInput>,
}

#[derive(GraphQLInputObject, Serialize, Debug, Clone, PartialEq)]
#[graphql(description = "Delete attribute input object")]
pub struct DeleteAttributeInput {
    #[graphql(description = "Client mutation id.")]
    #[serde(skip_serializing)]
    pub client_mutation_id: String,
    #[graphql(description = "Id of a attribute.")]
    #[serde(skip_serializing)]
    pub id: GraphqlID,
}

#[derive(GraphQLInputObject, Serialize, Debug, Clone)]
#[graphql(description = "Create attribute value input object")]
pub struct CreateAttributeValueInput {
    #[graphql(description = "Client mutation id.")]
    #[serde(skip_serializing)]
    pub client_mutation_id: String,
    #[graphql(description = "Attribute id.")]
    #[serde(skip_serializing)]
    pub raw_attribute_id: i32,
    #[graphql(description = "Attribute value code.")]
    pub code: String,
    #[graphql(description = "Attribute value translations.")]
    pub translations: Option<Vec<TranslationInput>>,
}

#[derive(GraphQLInputObject, Serialize, Debug, Clone)]
#[graphql(description = "Create attribute value input object")]
pub struct CreateAttributeValueWithAttributeInput {
    #[graphql(description = "Attribute value code.")]
    pub code: String,
    #[graphql(description = "Attribute value translations.")]
    pub translations: Option<Vec<TranslationInput>>,
}

#[derive(GraphQLInputObject, Serialize, Debug, Clone, PartialEq)]
#[graphql(description = "Update attribute value input object")]
pub struct UpdateAttributeValueInput {
    #[graphql(description = "Client mutation id.")]
    #[serde(skip_serializing)]
    pub client_mutation_id: String,
    #[graphql(description = "Attribute id.")]
    #[serde(skip_serializing)]
    pub raw_attribute_id: i32,
    #[graphql(description = "Attribute Value id.")]
    #[serde(skip_serializing)]
    pub raw_id: i32,
    #[graphql(description = "Attribute value code.")]
    pub code: Option<String>,
    #[graphql(description = "Attribute value translations.")]
    pub translations: Option<Vec<TranslationInput>>,
}

#[derive(GraphQLInputObject, Serialize, Debug, Clone)]
#[graphql(description = "Update attribute value input object")]
pub struct DeleteAttributeValueInput {
    #[graphql(description = "Client mutation id.")]
    #[serde(skip_serializing)]
    pub client_mutation_id: String,
    #[graphql(description = "Attribute Value id.")]
    #[serde(skip_serializing)]
    pub raw_id: i32,
}

impl UpdateAttributeInput {
    pub fn is_none(&self) -> bool {
        Self {
            client_mutation_id: self.client_mutation_id.clone(),
            id: self.id.clone(),
            name: None,
            meta_field: None,
        } == self.clone()
    }
}

impl UpdateAttributeValueInput {
    pub fn is_none(&self) -> bool {
        &Self {
            client_mutation_id: self.client_mutation_id.clone(),
            raw_attribute_id: self.raw_attribute_id.clone(),
            raw_id: self.raw_id.clone(),
            code: None,
            translations: None,
        } == self
    }
}

#[derive(GraphQLInputObject, Serialize, Debug, Clone)]
#[graphql(description = "Create attribute input object")]
pub struct CreateAttributeInput {
    #[graphql(name = "clientMutationId", description = "Client mutation id.")]
    pub uuid: String,
    #[graphql(description = "Name of an attribute.")]
    pub name: Vec<TranslationInput>,
    #[graphql(description = "Attribute type")]
    pub value_type: AttributeType,
    #[graphql(description = "Meta field of an attribute.")]
    pub meta_field: Option<AttributeMetaFieldInput>,
    #[graphql(description = "Attribute values.")]
    pub values: Option<Vec<CreateAttributeValueWithAttributeInput>>,
}

impl CreateAttributeInput {
    pub fn fill_uuid(mut self) -> Self {
        self.uuid = Some(self.uuid)
            .filter(|id| !id.is_empty())
            .unwrap_or_else(|| Uuid::new_v4().hyphenated().to_string());
        self
    }
}

#[derive(GraphQLInputObject, Deserialize, Serialize, Debug, Clone, PartialEq)]
#[graphql(name = "ProdAttrValueInput", description = "Product attributes with values input object")]
pub struct ProdAttrValueInput {
    #[graphql(description = "Int Attribute id")]
    pub attr_id: i32,
    #[graphql(description = "Attribute value")]
    pub value: String,
    #[graphql(description = "Meta field")]
    pub meta_field: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ProdAttrValue {
    pub attr_id: AttributeId,
    pub attr_value_id: Option<AttributeValueId>,
    pub value: AttributeValueCode,
    pub meta_field: Option<String>,
}

#[derive(GraphQLInputObject, Serialize, Deserialize, Clone, Debug)]
#[graphql(description = "Attribute Filter")]
pub struct AttributeFilterInput {
    #[graphql(description = "Int Attribute id")]
    pub id: i32,
    #[graphql(description = "Values to be equal")]
    pub equal: Option<EqualFilterInput>,
    #[graphql(description = "Range values to compare")]
    pub range: Option<RangeFilterInput>,
}

#[derive(GraphQLInputObject, Serialize, Deserialize, Clone, Debug)]
#[graphql(description = "Equality Filter input")]
pub struct EqualFilterInput {
    #[graphql(description = "Values to be equal")]
    pub values: Vec<String>,
}

#[derive(GraphQLInputObject, Serialize, Deserialize, Clone, Debug)]
#[graphql(description = "Range Filter input")]
pub struct RangeFilterInput {
    #[graphql(description = "Min value")]
    pub min_value: Option<f64>,
    #[graphql(description = "Max value")]
    pub max_value: Option<f64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AttributeFilter {
    pub id: i32,
    pub equal: Option<EqualFilter>,
    pub range: Option<RangeFilter>,
}

#[derive(GraphQLObject, Serialize, Deserialize, Clone, Debug)]
#[graphql(description = "Equality Filter input")]
pub struct EqualFilter {
    #[graphql(description = "Values to be equal")]
    pub values: Vec<String>,
}

#[derive(GraphQLObject, Serialize, Deserialize, Clone, Debug)]
#[graphql(description = "Range Filter input")]
pub struct RangeFilter {
    #[graphql(description = "Min value")]
    pub min_value: Option<f64>,
    #[graphql(description = "Max value")]
    pub max_value: Option<f64>,
}
