use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use std::env;

#[derive(Debug, Clone)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from: String,
    pub use_tls: bool,
}

impl SmtpConfig {
    pub fn from_env() -> Option<Self> {
        let host = env::var("SMTP_HOST").ok()?;
        if host.trim().is_empty() {
            return None;
        }
        let port: u16 = env::var("SMTP_PORT")
            .unwrap_or_else(|_| "587".into())
            .parse()
            .unwrap_or(587);
        let username = env::var("SMTP_USER").unwrap_or_default();
        let password = env::var("SMTP_PASS").unwrap_or_default();
        let from = env::var("SMTP_FROM")
            .unwrap_or_else(|_| "Tooth Plus <noreply@toothplus.com.br>".into());
        let use_tls = env::var("SMTP_TLS")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(true);

        Some(Self {
            host,
            port,
            username,
            password,
            from,
            use_tls,
        })
    }
}

pub async fn send_otp_email(
    config: &SmtpConfig,
    to_email: &str,
    recipient_name: &str,
    clinic_name: &str,
    otp_code: &str,
) -> Result<(), String> {
    let email_body_html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; background-color: #f8fafc; color: #0f172a; margin: 0; padding: 20px; }}
        .card {{ max-width: 520px; margin: 0 auto; background: #ffffff; border-radius: 16px; padding: 32px; border: 1px solid #e2e8f0; box-shadow: 0 4px 12px rgba(0,0,0,0.05); }}
        .header {{ border-bottom: 2px solid #0052cc; padding-bottom: 16px; margin-bottom: 24px; }}
        .clinic {{ font-size: 20px; font-weight: 800; color: #0052cc; margin: 0; }}
        .title {{ font-size: 14px; color: #64748b; margin-top: 4px; }}
        .pin-box {{ background: #f0fdf4; border: 2px dashed #10b981; border-radius: 12px; padding: 20px; text-align: center; margin: 24px 0; }}
        .pin-code {{ font-size: 32px; font-weight: 900; letter-spacing: 8px; color: #166534; font-family: monospace; }}
        .footer {{ font-size: 12px; color: #94a3b8; margin-top: 24px; text-align: center; }}
    </style>
</head>
<body>
    <div class="card">
        <div class="header">
            <h1 class="clinic">{clinic_name}</h1>
            <div class="title">Validação de Assinatura Digital (Lei 14.063/2020)</div>
        </div>
        <p>Olá <strong>{recipient_name}</strong>,</p>
        <p>Você solicitou a validação de segurança para assinatura de seu documento clínico.</p>
        <div class="pin-box">
            <div style="font-size: 12px; font-weight: bold; color: #15803d; text-transform: uppercase; margin-bottom: 6px;">Seu Código de Confirmação</div>
            <div class="pin-code">{otp_code}</div>
        </div>
        <p style="font-size: 13px; color: #475569;">Este código é válido por <strong>5 minutos</strong>. Não compartilhe com terceiros.</p>
        <div class="footer">
            Documento assinado com segurança criptográfica pela plataforma Tooth Plus.
        </div>
    </div>
</body>
</html>"#
    );

    let email = Message::builder()
        .from(
            config
                .from
                .parse()
                .map_err(|e| format!("Invalid from address: {}", e))?,
        )
        .to(to_email
            .parse()
            .map_err(|e| format!("Invalid to address: {}", e))?)
        .subject(format!("🔐 Código de Assinatura Digital - {}", clinic_name))
        .header(ContentType::TEXT_HTML)
        .body(email_body_html)
        .map_err(|e| format!("Failed to build email message: {}", e))?;

    let mut builder = AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&config.host)
        .port(config.port);

    if !config.username.is_empty() && !config.password.is_empty() {
        let creds = Credentials::new(config.username.clone(), config.password.clone());
        builder = builder.credentials(creds);
    }

    let transport = builder.build();

    transport
        .send(email)
        .await
        .map_err(|e| format!("SMTP send failed: {}", e))?;

    Ok(())
}
