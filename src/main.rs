pub mod tf_luna;
pub mod ld2410c;

#[tokio::main]
async fn main() {
    let mut tf_luna = tf_luna::TfLuna::new("COM8".to_string());
    tf_luna.connect().await.unwrap();
    tf_luna.set_output_frequency(tf_luna::OutputFrequency::Freq10Hz).await;
    tf_luna.set_output_format_setting(tf_luna::OutputFormat::NineByteCm).await;
    tf_luna.set_distance_limit_setting(0, 800).await;
    loop {
        tf_luna.read_data().await.unwrap();
        println!("Get configuration");
        tf_luna.get_configuration(tf_luna::OutputMode::Frequency).await;
        println!("--------------------------");

    }
}





