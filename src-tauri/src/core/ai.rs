use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize)]
pub struct AIResponse {
    pub suggested_actions: Vec<String>,
    pub params: serde_json::Value,
    pub explanation: String,
}

pub struct AIService {
    api_key: String,
    base_url: String,
}

impl AIService {
    pub fn new(api_key: String, base_url: String) -> Self {
        Self { api_key, base_url }
    }

    pub async fn test_connection(&self) -> Result<String> {
        let client = reqwest::Client::new();
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        
        let response = client.post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&json!({
                "model": "deepseek-chat",
                "messages": [{"role": "user", "content": "Hello, are you online?"}],
                "max_tokens": 10
            }))
            .send()
            .await?;

        if response.status().is_success() {
            Ok("连接成功！API Key 有效。".to_string())
        } else {
            let error_text = response.text().await?;
            Err(anyhow::anyhow!("连接失败: {}", error_text))
        }
    }

    pub async fn analyze_requirement(&self, prompt: &str) -> Result<AIResponse> {
        let client = reqwest::Client::new();
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        
        let system_prompt = r#"
You are a video processing AI assistant. Your goal is to analyze the user's requirement and suggest a list of video processing actions and their parameters.
The available actions are:
- Basic: cut (cut seconds), mirror (flip), rotate (rotate angle), speed (speed range), fps (target fps), bitrate (target bitrate)
- Visual: sharpen, denoise, blur, grain, vignette, border, portrait (soft focus), color (temp), pull (border), progressive (frame drop), corner (blur)
- AI/Effects: zoom, dissolve, scan, bounce, trifold, flash, lava, noise, pitch

Output MUST be a valid JSON object with the following structure:
{
    "suggested_actions": ["action_id1", "action_id2"],
    "params": {
        "param_name1": value1,
        "param_name2": value2
    },
    "explanation": "Brief explanation of why these actions were chosen."
}

Example params:
- cut_seconds: 1.0-10.0
- rotate_angle: 0.1-10.0
- speed_range: 0.01-0.5
- sharpen_strength: 0.0-5.0
- denoise_strength: 0.0-10.0
- blur_strength: 0.0-10.0
- grain_strength: 0.0-1.0
- vignette_strength: 0.0-1.0
- border_width: 10-100
- portrait_strength: 0.0-10.0
- color_temp_range: 100-2000
- pull_width: 10-200
- progressive_ratio: 0.0-0.5
- corner_radius: 10-200
- zoom_range: 0.0-0.5
- dissolve_strength: 0.0-1.0
- scan_strength: 0.0-1.0
- bounce_amplitude: 0.0-100.0
- trifold_spacing: 0-50
- flash_strength: 0.0-1.0
- lava_strength: 0.0-1.0
- noise_strength: 0.0-0.1
- pitch_range: 0.0-12.0

Do not include markdown formatting (like ```json). Just return the raw JSON string.
"#;

        let response = client.post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&json!({
                "model": "deepseek-chat",
                "messages": [
                    {"role": "system", "content": system_prompt},
                    {"role": "user", "content": prompt}
                ],
                "temperature": 0.7
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("API 请求失败: {}", error_text));
        }

        let response_json: serde_json::Value = response.json().await?;
        let content = response_json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("无法解析 API 响应内容"))?;

        // Clean up markdown code blocks if present
        let clean_content = content.trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        let ai_response: AIResponse = serde_json::from_str(clean_content)
            .map_err(|e| anyhow::anyhow!("JSON 解析失败: {} \n原始内容: {}", e, content))?;

        Ok(ai_response)
    }
    pub async fn analyze_video_metadata(&self, metadata_summary: &str) -> Result<AIResponse> {
        let client = reqwest::Client::new();
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        
        let system_prompt = r#"
You are a professional video engineer AI. Your job is to analyze video metadata summaries and suggest optimal processing parameters.
Based on the provided video characteristics (resolution, duration, bitrate, etc.), suggest a set of actions to improve or stylize the videos.

Rules:
1. If videos are vertical (9:16) and short, consider them as TikTok/Shorts style -> suggest fast pacing, sharpening, and color boosting.
2. If videos are horizontal (16:9), consider them as Cinematic/Vlog -> suggest cinematic bars, grading, etc.
3. If bitrate is low, suggest denoising and sharpening.
4. If duration is long, suggest simple cuts or speed ups.

The available actions and output format MUST be exactly the same as the standard requirements:
{
    "suggested_actions": ["action_id1", "action_id2"],
    "params": { ... },
    "explanation": "Brief explanation focused on the video characteristics."
}
Do not include markdown formatting.
"#;

        let response = client.post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&json!({
                "model": "deepseek-chat",
                "messages": [
                    {"role": "system", "content": system_prompt},
                    {"role": "user", "content": metadata_summary}
                ],
                "temperature": 0.7
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("API 请求失败: {}", error_text));
        }

        let response_json: serde_json::Value = response.json().await?;
        let content = response_json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("无法解析 API 响应内容"))?;

        let clean_content = content.trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        let ai_response: AIResponse = serde_json::from_str(clean_content)
            .map_err(|e| anyhow::anyhow!("JSON 解析失败: {} \n原始内容: {}", e, content))?;

        Ok(ai_response)
    }
}
