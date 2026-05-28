use super::schema;
use serde::Deserialize;
use shopify_function::prelude::*;
use shopify_function::Result;

// 1. Use the same configuration struct structure (from cart-transform-extension)
#[derive(Deserialize, Debug)]
struct FlowerArrangement {
    flower: Vec<String>,
    arrangement: String,
    color: String,
}

#[shopify_function]
fn cart_validations_generate_run(
    input: schema::cart_validations_generate_run::Input,
) -> Result<schema::CartValidationsGenerateRunResult> {
    let mut operations = Vec::new();
    let mut errors = Vec::new();

    // Loop through each item in the cart to check the flower selection boundaries
    for line in input.cart().lines() {
        // 2. Safely extract the custom configuration string attribute
        if let Some(config_str) = line.config().and_then(|attr| attr.value()) {
            // 3. Attempt to deserialize the configuration
            if let Ok(arrangement_data) = serde_json::from_str::<FlowerArrangement>(config_str) {
                let flower_count = arrangement_data.flower.len();

                // 4. Validate boundaries: Must be between 1 and 3 flowers inclusive
                if flower_count == 0 {
                    errors.push(schema::ValidationError {
                        message: "Choose between 1 to 3 flowers. You have chosen none.".to_owned(),
                        // Target the specific cart line ID so the error renders nicely in the UI
                        target: format!("$.cart.lines[{}]", line.id()),
                    });
                } else if flower_count > 3 {
                    errors.push(schema::ValidationError {
                        message: format!(
                            "Choose between 1 to 3 flowers. You have chosen {}.",
                            flower_count
                        ),
                        target: format!("$.cart.lines[{}]", line.id()),
                    });
                }
            }
        }
    }

    // 5. Wrap and return validation results to Shopify Core
    if !errors.is_empty() {
        let operation = schema::ValidationAddOperation { errors };
        operations.push(schema::Operation::ValidationAdd(operation));
    }

    Ok(schema::CartValidationsGenerateRunResult { operations })
}
