import { defineConfig, globalIgnores } from "eslint/config";
import nextVitals from "eslint-config-next/core-web-vitals";
import nextTs from "eslint-config-next/typescript";
import prettier from "eslint-config-prettier/flat";

const eslintConfig = defineConfig([
    ...nextVitals,
    ...nextTs,
    prettier,
    // Override default ignores of eslint-config-next.
    globalIgnores([
        // Default ignores of eslint-config-next:
        ".next/**",
        "out/**",
        "build/**",
        "next-env.d.ts",
        // Rust build artifacts - not part of our codebase
        "target/**",
        "src-tauri/target/**",
        "crates/**/target/**",
        // Python
        "crates/lex-learning/runtime",
        "crates/lex-learning/python",
    ]),
]);

export default eslintConfig;
