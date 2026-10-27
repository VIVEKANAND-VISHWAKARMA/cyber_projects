#[derive(Debug, Clone)]
pub struct Classification {
    pub categories: Vec<String>,
    pub confidence: f32,
    pub evidence: Vec<String>,
}

struct Rule {
    category: &'static str,
    keywords: &'static [&'static str],
}

const RULES: &[Rule] = &[
    Rule {
        category: "Active Directory",
        keywords: &[
            "active directory",
            "active-directory",
            "adcs",
            "bloodhound",
            "domain controller",
        ],
    },
    Rule {
        category: "Kerberos",
        keywords: &[
            "kerberos",
            "kerberoasting",
            "as-rep roasting",
            "golden ticket",
            "silver ticket",
        ],
    },
    Rule {
        category: "Windows",
        keywords: &["windows", "powershell", "winrm", "smb", "ntlm", "mimikatz"],
    },
    Rule {
        category: "Linux",
        keywords: &["linux", "bash", "sudo", "cron", "suid", "systemd"],
    },
    Rule {
        category: "Privilege Escalation",
        keywords: &[
            "privilege escalation",
            "priv esc",
            "privesc",
            "local escalation",
        ],
    },
    Rule {
        category: "Web Exploitation",
        keywords: &[
            "web exploitation",
            "web application",
            "sql injection",
            "xss",
            "ssrf",
            "file inclusion",
            "idor",
            "directory traversal",
        ],
    },
    Rule {
        category: "Networking & Enumeration",
        keywords: &[
            "nmap",
            "network scanning",
            "enumeration",
            "dns",
            "networking",
            "port scanning",
        ],
    },
    Rule {
        category: "Cryptography",
        keywords: &[
            "cryptography",
            "crypto",
            "rsa",
            "aes",
            "cipher",
            "hashing",
            "encoding",
        ],
    },
    Rule {
        category: "Digital Forensics",
        keywords: &[
            "forensics",
            "memory analysis",
            "disk image",
            "volatility",
            "autopsy",
            "incident response",
        ],
    },
    Rule {
        category: "Reverse Engineering",
        keywords: &[
            "reverse engineering",
            "disassembly",
            "decompilation",
            "ghidra",
            "ida pro",
        ],
    },
    Rule {
        category: "Binary Exploitation",
        keywords: &[
            "buffer overflow",
            "rop chain",
            "pwntools",
            "pwn",
            "format string",
            "heap exploitation",
        ],
    },
    Rule {
        category: "Cloud Security",
        keywords: &[
            "cloud security",
            "aws",
            "azure",
            "gcp",
            "iam policy",
            "s3 bucket",
        ],
    },
    Rule {
        category: "Malware Analysis",
        keywords: &[
            "malware analysis",
            "ransomware",
            "static analysis",
            "dynamic analysis",
            "yara",
        ],
    },
    Rule {
        category: "Wireless Security",
        keywords: &["wireless", "wifi", "wi-fi", "wpa2", "wireless security"],
    },
];

/// Classifies a room using its name, description, tags, and learning objectives.
///
/// Supply all available metadata in `text`. The classifier uses deterministic
/// keyword rules and may return multiple categories.
pub fn classify(text: &str) -> Classification {
    let normalized = text.to_lowercase();
    let mut categories = Vec::new();
    let mut evidence = Vec::new();

    for rule in RULES {
        for keyword in rule.keywords {
            if normalized.contains(keyword) {
                if !categories.iter().any(|c| c == rule.category) {
                    categories.push(rule.category.to_string());
                }

                evidence.push(format!("{}: {}", rule.category, keyword));
            }
        }
    }

    let confidence = match categories.len() {
        0 => 0.0,
        1 => 0.75,
        2 => 0.85,
        _ => 0.90,
    };

    Classification {
        categories,
        confidence,
        evidence,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_active_directory_and_kerberos() {
        let result = classify("Active Directory attacks covering Kerberos and Kerberoasting");

        assert!(result.categories.contains(&"Active Directory".to_string()));
        assert!(result.categories.contains(&"Kerberos".to_string()));
    }

    #[test]
    fn unknown_text_has_no_categories() {
        let result = classify("Cool");

        assert!(result.categories.is_empty());
        assert_eq!(result.confidence, 0.0);
    }

    #[test]
    fn classifies_web_exploitation() {
        let result = classify("SQL injection in a web application");

        assert!(result.categories.contains(&"Web Exploitation".to_string()));
    }
}
