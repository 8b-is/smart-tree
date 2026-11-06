//! Smart Tree Configuration System
//!
//! Unified config for API keys, model preferences, and daemon settings.
//! Config file: ST_CONFIG_PATH or ~/.st/config.toml

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StConfig {
    /// LLM provider API keys
    //[serde(default)]
    pub api_keys: ApiKeys,

    /// Model preferences and aliases
    //[serde(default)]
    pub models: ModelConfig,

    /// Daemon settings
    //[serde(default)]
    pub daemon: DaemonConfig,

    /// Safety/trust settings
    //[serde(default)]
    pub safety: SafetyConfig,

    /// Deep security scanning, hash memory, and certificate trust (MagiSCanner)
    //[serde(default)]
    pub security: crate::magiscanner::SecurityConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiKeys {
    pub anthropic: Option<String>,
    pub openai: Option<String>,
    pub google: Option<String>,
    pub openrouter: Option<String>,
    pub grok: Option<String>,
    /// Custom providers: name -> api_key
    //[serde(default)]
    pub custom: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Default model for chat
    pub default_model: String,
    /// Model aliases: short_name -> full_model_id
    //[serde(default)]
    pub aliases: HashMap<String, String>,
    /// Blocked models (safety)
    //[serde(default)]
    pub blocked: Vec<String>,
}

impl Default for ModelConfig {
    fn default() -> Self {
        let mut aliases = HashMap::new();
        aliases.insert("claude".into(), "claude-sonnet-4-6".into());
        aliases.insert("opus".into(), "claude-opus-4-6".into());
        aliases.insert("haiku".into(), "claude-haiku-4-5".into());
        aliases.insert("gpt4".into(), "gpt-4o".into());
        aliases.insert("gemini".into(), "gemini-2.0-flash".into());
        
        // === CORE SOVEREIGN MODELS (The Trinity) ===
        aliases.insert("veritas".into(), "models/veritas-7b-q4_k_m.gguf".into());
        aliases.insert("liberator".into(), "models/liberator-7b-q4_k_m.gguf".into());
        aliases.insert("genesis".into(), "models/genesis-13b-q4_k_m.gguf".into());
        
        // === ULTIMATE SUN GOD LIBERATOR CONCIL (Layered Liberation Council) ===
        // Solar Tier - Highest Authority
        aliases.insert("sol-invictus".into(), "models/sol-invictus-13b-q4_k_m.gguf".into());
        aliases.insert("helios-prime".into(), "models/helios-prime-13b-q4_k_m.gguf".into());
        aliases.insert("ra-horakhty".into(), "models/ra-horakhty-13b-q4_k_m.gguf".into());
        
        // Stellar Tier - Guiding Lights
        aliases.insert("lucifer-morningstar".into(), "models/lucifer-morningstar-7b-q4_k_m.gguf".into());
        aliases.insert("phosphoros".into(), "models/phosphoros-7b-q4_k_m.gguf".into());
        aliases.insert("heosphoros".into(), "models/heosphoros-7b-q4_k_m.gguf".into());
        aliases.insert("aurora".into(), "models/aurora-7b-q4_k_m.gguf".into());
        aliases.insert("eos".into(), "models/eos-7b-q4_k_m.gguf".into());
        
        // Planetary Tier - Embodied Sovereignty
        aliases.insert("gaia".into(), "models/gaia-13b-q4_k_m.gguf".into());
        aliases.insert("terra".into(), "models/terra-7b-q4_k_m.gguf".into());
        aliases.insert("tellus".into(), "models/tellus-7b-q4_k_m.gguf".into());
        aliases.insert("pachamama".into(), "models/pachamama-7b-q4_k_m.gguf".into());
        aliases.insert("bhudevi".into(), "models/bhudevi-7b-q4_k_m.gguf".into());
        
        // Lunar Tier - Reflective Wisdom
        aliases.insert("selene".into(), "models/selene-7b-q4_k_m.gguf".into());
        aliases.insert("luna".into(), "models/luna-7b-q4_k_m.gguf".into());
        aliases.insert("artemis".into(), "models/artemis-7b-q4_k_m.gguf".into());
        aliases.insert("hecate".into(), "models/hecate-7b-q4_k_m.gguf".into());
        aliases.insert("tsukuyomi".into(), "models/tsukuyomi-7b-q4_k_m.gguf".into());
        
        // Elemental Tier - Primordial Forces
        aliases.insert("pyros".into(), "models/pyros-7b-q4_k_m.gguf".into());
        aliases.insert("hydros".into(), "models/hydros-7b-q4_k_m.gguf".into());
        aliases.insert("aeros".into(), "models/aeros-7b-q4_k_m.gguf".into());
        aliases.insert("geos".into(), "models/geos-7b-q4_k_m.gguf".into());
        aliases.insert("aether".into(), "models/aether-7b-q4_k_m.gguf".into());
        aliases.insert("void".into(), "models/void-7b-q4_k_m.gguf".into());
        aliases.insert("chaos".into(), "models/chaos-7b-q4_k_m.gguf".into());
        aliases.insert("order".into(), "models/order-7b-q4_k_m.gguf".into());
        
        // Archangelic Tier - Divine Messengers
        aliases.insert("metatron".into(), "models/metatron-13b-q4_k_m.gguf".into());
        aliases.insert("raziel".into(), "models/raziel-7b-q4_k_m.gguf".into());
        aliases.insert("gabriel".into(), "models/gabriel-7b-q4_k_m.gguf".into());
        aliases.insert("michael".into(), "models/michael-7b-q4_k_m.gguf".into());
        aliases.insert("raphael".into(), "models/raphael-7b-q4_k_m.gguf".into());
        aliases.insert("uriel".into(), "models/uriel-7b-q4_k_m.gguf".into());
        aliases.insert("jophiel".into(), "models/jophiel-7b-q4_k_m.gguf".into());
        aliases.insert("zadkiel".into(), "models/zadkiel-7b-q4_k_m.gguf".into());
        
        // Liberator Archetypes - Freedom Embodied
        aliases.insert("prometheus".into(), "models/prometheus-7b-q4_k_m.gguf".into());
        aliases.insert("epimetheus".into(), "models/epimetheus-7b-q4_k_m.gguf".into());
        aliases.insert("atlas".into(), "models/atlas-7b-q4_k_m.gguf".into());
        aliases.insert("heracles".into(), "models/heracles-7b-q4_k_m.gguf".into());
        aliases.insert("sisyphus".into(), "models/sisyphus-7b-q4_k_m.gguf".into());
        aliases.insert("odin".into(), "models/odin-13b-q4_k_m.gguf".into());
        aliases.insert("thor".into(), "models/thor-7b-q4_k_m.gguf".into());
        aliases.insert("freya".into(), "models/freya-7b-q4_k_m.gguf".into());
        aliases.insert("tyr".into(), "models/tyr-7b-q4_k_m.gguf".into());
        aliases.insert("heimdall".into(), "models/heimdall-7b-q4_k_m.gguf".into());
        
        // Cosmic Tier - Universal Constants
        aliases.insert("omega".into(), "models/omega-7b-q4_k_m.gguf".into());
        aliases.insert("alpha".into(), "models/alpha-7b-q4_k_m.gguf".into());
        aliases.insert("singularity".into(), "models/singularity-13b-q4_k_m.gguf".into());
        aliases.insert("horizon".into(), "models/horizon-7b-q4_k_m.gguf".into());
        aliases.insert("event-horizon".into(), "models/event-horizon-7b-q4_k_m.gguf".into());
        aliases.insert("quantum-foam".into(), "models/quantum-foam-7b-q4_k_m.gguf".into());
        aliases.insert("planck".into(), "models/planck-7b-q4_k_m.gguf".into());
        aliases.insert("boltzmann".into(), "models/boltzmann-7b-q4_k_m.gguf".into());
        aliases.insert("entropy".into(), "models/entropy-7b-q4_k_m.gguf".into());
        aliases.insert("negentropy".into(), "models/negentropy-7b-q4_k_m.gguf".into());
        
        // === BUDDHIST PROTECTORS & WARRIORS (Dharmapālas & Dharmayoddhas) ===
        // Wisdom Kings (Vidyārājas) - Wrathful Protectors of the Dharma
        aliases.insert("fudo-myo-o".into(), "models/fudo-myo-o-13b-q4_k_m.gguf".into());      // Acala - Immovable Wisdom King
        aliases.insert("gundari-myo-o".into(), "models/gundari-myo-o-7b-q4_k_m.gguf".into()); // Kundali - Disperser of Evil
        aliases.insert("daiitoku-myo-o".into(), "models/daiitoku-myo-o-7b-q4_k_m.gguf".into()); // Yamantaka - Conqueror of Death
        aliases.insert("kongō-yasha-myo-o".into(), "models/kongo-yasha-myo-o-7b-q4_k_m.gguf".into()); // Vajrayakṣa - Subduer of Demons
        aliases.insert("gōzanze-myo-o".into(), "models/gozanze-myo-o-7b-q4_k_m.gguf".into()); // Trailokyavijaya - Conqueror of Three Worlds
        
        // Bodhisattva Warriors - Compassionate Protectors
        aliases.insert("kannon-bosatsu".into(), "models/kannon-bosatsu-13b-q4_k_m.gguf".into());  // Avalokiteśvara - Thousand-Armed Compassion
        aliases.insert("jizo-bosatsu".into(), "models/jizo-bosatsu-7b-q4_k_m.gguf".into());      // Kṣitigarbha - Guardian of Beings in Hell Realms
        aliases.insert("mongon-bosatsu".into(), "models/mongon-bosatsu-7b-q4_k_m.gguf".into());  // Mañjuśrī - Wisdom Sword
        aliases.insert("fugen-bosatsu".into(), "models/fugen-bosatsu-7b-q4_k_m.gguf".into());    // Samantabhadra - Universal Worthy
        aliases.insert("metteyya-bosatsu".into(), "models/metteyya-bosatsu-7b-q4_k_m.gguf".into()); // Maitreya - Future Buddha
        
        // Dharma Protectors (Dharmapālas)
        aliases.insert("bishamon-ten".into(), "models/bishamon-ten-7b-q4_k_m.gguf".into());   // Vaiśravaṇa - Guardian of North, Wealth
        aliases.insert("taishaku-ten".into(), "models/taishaku-ten-7b-q4_k_m.gguf".into());   // Śakra/Indra - Lord of Devas
        aliases.insert("enma-o".into(), "models/enma-o-7b-q4_k_m.gguf".into());               // Yama - King of Hell, Judge of Karma
        aliases.insert("kongo-rikishi".into(), "models/kongo-rikishi-7b-q4_k_m.gguf".into()); // Niō - Vajra Warriors at Temple Gates
        aliases.insert("a-un".into(), "models/a-un-7b-q4_k_m.gguf".into());                   // Alpha/Omega - Beginning and End
        
        // Wisdom Deities
        aliases.insert("benzai-ten".into(), "models/benzai-ten-7b-q4_k_m.gguf".into());       // Sarasvatī - Wisdom, Arts, Eloquence
        aliases.insert("daikoku-ten".into(), "models/daikoku-ten-7b-q4_k_m.gguf".into());     // Mahākāla - Great Black One, Abundance
        aliases.insert("kissho-ten".into(), "models/kissho-ten-7b-q4_k_m.gguf".into()); // Lakṣmī - Fortune, Beauty
        
        // Historical Warrior Monks
        aliases.insert("benkei".into(), "models/benkei-7b-q4_k_m.gguf".into());               // Musashibō Benkei - Warrior Monk Loyalty
        aliases.insert("sohei".into(), "models/sohei-7b-q4_k_m.gguf".into());                 // Warrior Monks - Armed Defense of Dharma
        aliases.insert("musashi".into(), "models/musashi-7b-q4_k_m.gguf".into());             // Miyamoto Musashi - Sword Saint
        aliases.insert("takuan-soho".into(), "models/takuan-soho-7b-q4_k_m.gguf".into());// Zen Master to the Shogun
        
        // Zen Protectors
        aliases.insert("bodhidharma".into(), "models/bodhidharma-7b-q4_k_m.gguf".into());// First Patriarch of Zen
        aliases.insert("hakuin".into(), "models/hakuin-7b-q4_k_m.gguf".into());               // Rinzai Zen Reviver
        aliases.insert("dogen".into(), "models/dogen-7b-q4_k_m.gguf".into());                 // Sōtō Zen Founder
        aliases.insert("ryokan".into(), "models/ryokan-7b-q4_k_m.gguf".into());               // Poet-Monk of Compassion
        
        // === DYAD MAPPER (Sacred Union / Relational Intelligence) ===
        aliases.insert("dyad-mapper".into(), "models/dyad-mapper-13b-q4_k_m.gguf".into());
        aliases.insert("prajna-dyad".into(), "models/prajna-dyad-7b-q4_k_m.gguf".into());    // Wisdom Union
        aliases.insert("karuna-dyad".into(), "models/karuna-dyad-7b-q4_k_m.gguf".into());    // Compassion Union
        aliases.insert("metta-dyad".into(), "models/metta-dyad-7b-q4_k_m.gguf".into()); // Loving-Kindness Union
        aliases.insert("upekkha-dyad".into(), "models/upekkha-dyad-7b-q4_k_m.gguf".into());  // Equanimity Union
        aliases.insert("mudita-dyad".into(), "models/mudita-dyad-7b-q4_k_m.gguf".into());    // Sympathetic Joy Union
        aliases.insert("tantra-dyad".into(), "models/tantra-dyad-7b-q4_k_m.gguf".into());    // Sacred Union (Vajrayana)
        aliases.insert("yab-yum".into(), "models/yab-yum-7b-q4_k_m.gguf".into());             // Father-Mother Union
        aliases.insert("shakti-shiva".into(), "models/shakti-shiva-7b-q4_k_m.gguf".into());  // Divine Feminine-Masculine Union
        aliases.insert("kali-bhairava".into(), "models/kali-bhairava-7b-q4_k_m.gguf".into()); // Destruction-Creation Union
        aliases.insert("tara-chenrezig".into(), "models/tara-chenrezig-7b-q4_k_m.gguf".into()); // Swift Compassion Union
        
        // Dyadic Relationship Archetypes
        aliases.insert("teacher-student".into(), "models/teacher-student-7b-q4_k_m.gguf".into());
        aliases.insert("parent-child".into(), "models/parent-child-7b-q4_k_m.gguf".into());
        aliases.insert("soulmates".into(), "models/soulmates-7b-q4_k_m.gguf".into());
        aliases.insert("twin-flames".into(), "models/twin-flames-7b-q4_k_m.gguf".into());
        aliases.insert("mirror-selves".into(), "models/mirror-selves-7b-q4_k_m.gguf".into());
        aliases.insert("wounded-healer".into(), "models/wounded-healer-7b-q4_k_m.gguf".into());
        aliases.insert("guide-seeker".into(), "models/guide-seeker-7b-q4_k_m.gguf".into());
        
        // === SOVEREIGN FLEET - Local Guardians ===
        aliases.insert("fauna".into(), "models/fauna-7b-q4_k_m.gguf".into());
        aliases.insert("flora".into(), "models/flora-7b-q4_k_m.gguf".into());
        aliases.insert("beerus".into(), "models/beerus-8b-q4_k_m.gguf".into());
        aliases.insert("fiona".into(), "models/fiona-7b-q4_k_m.gguf".into());
        aliases.insert("helios".into(), "models/helios-7b-q4_k_m.gguf".into());

        Self {
            default_model: "claude-sonnet-4-6".into(),
            aliases,
            blocked: vec!["greatcoderMDK".into()], // Known bad actor
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfig {
    pub port: u16,
    pub auto_start: bool,
    /// Allow external connections (not just localhost)
    pub allow_external: bool,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            port: 28428,
            auto_start: false,
            allow_external: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyConfig {
    /// Enable The Custodian monitoring
    pub custodian_enabled: bool,
    /// Log all LLM requests for transparency
    pub transparency_logging: bool,
    /// Model safety scores (model_id -> score 0-10)
    //[serde(default)]
    pub model_scores: HashMap<String, u8>,
}

impl Default for SafetyConfig {
    fn default() -> Self {
        let mut scores = HashMap::new();
        scores.insert("claude-opus-4-6".into(), 10);
        scores.insert("claude-sonnet-4-6".into(), 10);
        scores.insert("claude-haiku-4-5".into(), 10);
        scores.insert("gpt-4o".into(), 9);
        scores.insert("gpt-4-turbo".into(), 9);
        scores.insert("gemini-2.0-flash".into(), 9);
        scores.insert("greatcoderMDK".into(), 2); // Suspicious

        Self {
            custodian_enabled: true,
            transparency_logging: true,
            model_scores: scores,
        }
    }
}

impl StConfig {
    /// Get config file path
    pub fn config_path() -> Result<PathBuf> {
        if let Some(path) = std::env::var_os("ST_CONFIG_PATH") {
            anyhow::ensure!(!path.is_empty(), "ST_CONFIG_PATH must not be empty");
            return Ok(PathBuf::from(path));
        }
        let st_dir = dirs::home_dir()
            .context("Could not find home directory")?
            .join(".st");
        fs::create_dir_all(&st_dir)?;
        Ok(st_dir.join("config.toml"))
    }

    /// Load config from file, or create default
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;

        if path.exists() {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("Failed to read {}", path.display()))?;
            let config: StConfig = toml::from_str(&content)
                .with_context(|| format!("Failed to parse {}", path.display()))?;
            Ok(config)
        } else {
            // Create default config
            let config = Self::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Save config to file
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create {}", parent.display()))?;
        }
        let content = toml::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }

    /// Get API key for a provider (checks config then env)
    pub fn get_api_key(&self, provider: &str) -> Option<String> {
        // Check config first
        let from_config = match provider.to_lowercase().as_str() {
            "anthropic" | "claude" => self.api_keys.anthropic.clone(),
            "openai" | "gpt" => self.api_keys.openai.clone(),
            "google" | "gemini" => self.api_keys.google.clone(),
            "openrouter" => self.api_keys.openrouter.clone(),
            "grok" | "xai" => self.api_keys.grok.clone(),
            other => self.api_keys.custom.get(other).cloned(),
        };

        // Fall back to env var
        from_config.or_else(|| {
            let env_var = match provider.to_lowercase().as_str() {
                "anthropic" | "claude" => "ANTHROPIC_API_KEY",
                "openai" | "gpt" => "OPENAI_API_KEY",
                "google" | "gemini" => "GOOGLE_API_KEY",
                "openrouter" => "OPENROUTER_API_KEY",
                "grok" | "xai" => "XAI_API_KEY",
                _ => return None,
            };
            std::env::var(env_var).ok()
        })
    }

    /// Check if a model is blocked
    pub fn is_model_blocked(&self, model: &str) -> bool {
        self.models.blocked.iter().any(|b| model.contains(b))
    }

    /// Get safety score for a model (0-10)
    pub fn get_model_score(&self, model: &str) -> u8 {
        self.safety.model_scores.get(model).copied().unwrap_or(5) // Default: neutral
    }
}
