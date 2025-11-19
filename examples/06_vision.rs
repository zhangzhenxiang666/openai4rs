use dotenvy::dotenv;
use openai4rs::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let client = OpenAI::from_env()?;

    let model = "Qwen/Qwen2.5-VL-32B-Instruct";

    // Example image URL (replace with your own image URL or a local image encoded in base64)
    // For this example, we'll use a public image URL.
    // Note: Ensure the image URL is accessible and the model supports vision.
    let image_url = "https://qianwen-res.oss-cn-beijing.aliyuncs.com/QVQ/demo.png";

    let messages = vec![
        system!(content: "You are a helpful assistant that can describe images."),
        user!(content: vec![
            content!({"type": "image_url", "image_url":{"url":image_url}}) // This macro likely needs to be adapted for vision, but we'll use it for now.
            ,content!({"type": "text", "text": "What is in this image?"})
        ]),
    ];
    let request = ChatParam::new(model, &messages);
    println!("Sending request to model: {} with an image...", model);

    let response = client.chat().create(request).await?;

    if let Some(content) = response.content() {
        println!("\nImage Description:\n{}", content);
    } else {
        println!("\nNo description provided.");
    }

    Ok(())
}
