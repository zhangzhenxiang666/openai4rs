use dotenvy::dotenv;
use openai4rs::*;

// Mock function to get weather data
fn get_current_weather(location: &str, unit: Option<&str>) -> String {
    // In a real application, this would call an external weather API.
    let unit = unit.unwrap_or("celsius");
    format!(
        "The current weather in {} is 22 degrees {}.",
        location, unit
    )
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let client = OpenAI::from_env()?;

    let model = "Qwen/Qwen3-235B-A22B-Instruct-2507";

    // 1. Define the tool (function)
    let weather_tool_params = Parameters::object()
        .property(
            "location",
            Parameters::string()
                .description("The city and state, e.g. San Francisco, CA")
                .build(),
        )
        .property(
            "unit",
            Parameters::string()
                .description("The unit of temperature, e.g. celsius or fahrenheit")
                .build(),
        )
        .require("location")
        .build()?;

    let weather_tool = ChatCompletionToolParam::function(
        "get_current_weather",
        "Get the current weather in a given location",
        weather_tool_params,
    );

    // 2. Create the initial message and request
    let messages = vec![
        system!(content = "You are a helpful assistant."),
        user!(content = "What's the weather like in Boston today?"),
    ];

    let request = chat_request(model, &messages)
        .tools(vec![weather_tool])
        .tool_choice(ToolChoice::Auto)
        .build()?;

    println!("Sending request to model: {}...", model);

    let response = client.chat().create(request).await?;
    println!("Initial response: {:#?}", response);

    // 3. Check if the model wants to call a tool
    if response.has_tool_calls() {
        println!("\nModel wants to call a tool.");
        let tool_calls = response.tool_calls().unwrap();

        // For simplicity, we'll only handle the first tool call
        if let Some(tool_call) = tool_calls.first() {
            let function_name = &tool_call.function.name;
            let arguments_str = &tool_call.function.arguments;

            if function_name == "get_current_weather" {
                let args: serde_json::Value = serde_json::from_str(arguments_str)?;
                let location = args["location"].as_str().unwrap_or("Unknown");
                let unit = args["unit"].as_str();

                println!(
                    "Calling function '{}' with arguments: location='{}', unit='{:?}'",
                    function_name, location, unit
                );

                // 4. Call the function and get the result
                let function_result = get_current_weather(location, unit);
                println!("Function result: {}", function_result);

                // 5. Send the function result back to the model
                let mut new_messages = messages.clone();
                new_messages.push(response.first_choice_message().unwrap().clone().into());
                new_messages.push(tool!(
                    tool_call_id = tool_call.function.id.clone(),
                    content = function_result
                ));

                let follow_up_request = chat_request(model, &new_messages).build()?;

                let final_response = client.chat().create(follow_up_request).await?;
                if let Some(content) = final_response.content() {
                    println!("\nFinal Assistant Response:\n{}", content);
                }
            }
        }
    } else {
        // If no tool call, just print the content
        if let Some(content) = response.content() {
            println!("\nAssistant Response:\n{}", content);
        }
    }

    Ok(())
}
