@"
# Gent Flexible Staking Program

![Last Updated](https://img.shields.io/badge/Last%20Updated-2025--02--21-blue)
![License](https://img.shields.io/badge/License-MIT-green)

A flexible staking solution built on Solana using the Anchor framework, developed by Genterr.

## 🚀 Features

- Flexible staking periods
- Dynamic reward calculation
- Emergency withdrawal mechanism
- Real-time monitoring system
- Admin console for management

## 📋 Prerequisites

- Node.js 16.x or higher
- Rust and Cargo
- Solana CLI tools
- Anchor Framework

## 🛠 Installation

1. Clone the repository:
\`\`\`bash
git clone https://github.com/Genterr/Flexible-Staking.git
cd Flexible-Staking
\`\`\`

2. Install dependencies:
\`\`\`bash
npm install
\`\`\`

3. Configure environment:
\`\`\`bash
cp .env.example .env
\`\`\`

4. Update the \`.env\` file with your configuration:
- Replace \`SOLANA_RPC_URL\` with your Solana RPC endpoint
- Add your \`PROGRAM_ID\` after deployment
- Configure other variables as needed

## 💻 Development

Build the program:
\`\`\`bash
anchor build
\`\`\`

Run tests:
\`\`\`bash
anchor test
\`\`\`

Deploy:
\`\`\`bash
npm run deploy
\`\`\`

## 🔒 Security

⚠️ **Important**: 
- Never commit your \`.env\` file or any private keys
- Always use environment variables for sensitive data
- Review the \`.gitignore\` file to ensure sensitive files are excluded

## 🏗 Architecture

The staking program consists of several key components:

\`\`\`
├── programs/          # Solana programs (smart contracts)
├── app/              # Frontend application
├── tests/           # Program tests
├── scripts/         # Deployment and utility scripts
└── admin/           # Administrative tools
\`\`\`

## 🤝 Contributing

1. Fork the repository
2. Create your feature branch (\`git checkout -b feature/AmazingFeature\`)
3. Commit your changes (\`git commit -m 'Add some AmazingFeature'\`)
4. Push to the branch (\`git push origin feature/AmazingFeature\`)
5. Open a Pull Request

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 📞 Contact

Genterr - [@Genterr](https://github.com/Genterr)

Project Link: [https://github.com/Genterr/Flexible-Staking](https://github.com/Genterr/Flexible-Staking)

---
Last updated: 2025-02-21
"@ | Out-File -FilePath README.md -Encoding UTF8