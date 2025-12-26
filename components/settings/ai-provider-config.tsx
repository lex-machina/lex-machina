"use client";

import { useState, useCallback, useEffect } from "react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import type { AIProviderType, AIProviderConfig } from "@/types";
import type { ValidationStatus } from "@/lib/hooks/use-settings";

// ============================================================================
// TYPES
// ============================================================================

export interface AIProviderConfigProps {
  /** Current AI provider configuration (null if not configured) */
  config: AIProviderConfig | null;
  /** Callback to configure an AI provider */
  onConfigure: (provider: AIProviderType, apiKey: string) => Promise<void>;
  /** Callback to clear the AI provider */
  onClear: () => Promise<void>;
  /** Callback to validate an API key */
  onValidate: (provider: AIProviderType, apiKey: string) => Promise<boolean>;
  /** Current validation status */
  validationStatus: ValidationStatus;
  /** Validation error message */
  validationError: string | null;
  /** Whether the component is disabled */
  disabled?: boolean;
  /** Additional class names */
  className?: string;
}

// ============================================================================
// PROVIDER OPTIONS
// ============================================================================

interface ProviderOption {
  value: AIProviderType;
  label: string;
  description: string;
  placeholder: string;
  helpUrl: string;
  icon: React.ReactNode;
}

const PROVIDER_OPTIONS: ProviderOption[] = [
  {
    value: "openrouter",
    label: "OpenRouter",
    description: "Access multiple AI models through a single API",
    placeholder: "sk-or-v1-...",
    helpUrl: "https://openrouter.ai/keys",
    icon: <OpenRouterIcon />,
  },
  {
    value: "gemini",
    label: "Google Gemini",
    description: "Google's advanced AI model",
    placeholder: "AIza...",
    helpUrl: "https://aistudio.google.com/apikey",
    icon: <GeminiIcon />,
  },
];

// ============================================================================
// ICONS
// ============================================================================

function OpenRouterIcon() {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="20"
      height="20"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
      className="shrink-0"
    >
      <path d="M12 2L2 7l10 5 10-5-10-5Z" />
      <path d="m2 17 10 5 10-5" />
      <path d="m2 12 10 5 10-5" />
    </svg>
  );
}

function GeminiIcon() {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="20"
      height="20"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
      className="shrink-0"
    >
      <path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z" />
      <path d="M12 3a6 6 0 0 1-9 9 9 9 0 1 0 9-9Z" />
    </svg>
  );
}

function CheckCircleIcon() {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="16"
      height="16"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
      className="shrink-0"
    >
      <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" />
      <path d="m9 11 3 3L22 4" />
    </svg>
  );
}

function AlertCircleIcon() {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="16"
      height="16"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
      className="shrink-0"
    >
      <circle cx="12" cy="12" r="10" />
      <line x1="12" x2="12" y1="8" y2="12" />
      <line x1="12" x2="12.01" y1="16" y2="16" />
    </svg>
  );
}

function LoadingIcon() {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="16"
      height="16"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
      className="shrink-0 animate-spin"
    >
      <path d="M21 12a9 9 0 1 1-6.219-8.56" />
    </svg>
  );
}

function KeyIcon() {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="16"
      height="16"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
      className="shrink-0"
    >
      <circle cx="7.5" cy="15.5" r="5.5" />
      <path d="m21 2-9.6 9.6" />
      <path d="m15.5 7.5 3 3L22 7l-3-3" />
    </svg>
  );
}

function TrashIcon() {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="16"
      height="16"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
      className="shrink-0"
    >
      <path d="M3 6h18" />
      <path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6" />
      <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2" />
    </svg>
  );
}

function ExternalLinkIcon() {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="12"
      height="12"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
      className="shrink-0 ml-1"
    >
      <path d="M15 3h6v6" />
      <path d="M10 14 21 3" />
      <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" />
    </svg>
  );
}

// ============================================================================
// PROVIDER CARD COMPONENT
// ============================================================================

interface ProviderCardProps {
  option: ProviderOption;
  isSelected: boolean;
  isConfigured: boolean;
  onClick: () => void;
  disabled?: boolean;
}

function ProviderCard({
  option,
  isSelected,
  isConfigured,
  onClick,
  disabled,
}: ProviderCardProps) {
  return (
    <button
      type="button"
      onClick={onClick}
      disabled={disabled}
      className={cn(
        // Base styles
        "relative flex items-center gap-3 p-3 rounded-md text-left",
        "border transition-all duration-150",
        // Focus styles
        "focus:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background",
        // Selected state
        isSelected
          ? "border-primary bg-primary/5"
          : "border-border hover:border-muted-foreground/50 hover:bg-muted/50",
        // Disabled state
        disabled && "cursor-not-allowed opacity-50"
      )}
    >
      {/* Icon */}
      <div
        className={cn(
          "flex items-center justify-center w-10 h-10 rounded-md",
          isSelected
            ? "bg-primary text-primary-foreground"
            : "bg-muted text-muted-foreground"
        )}
      >
        {option.icon}
      </div>

      {/* Content */}
      <div className="flex flex-col gap-0.5 min-w-0 flex-1">
        <span className="text-sm font-medium">{option.label}</span>
        <span className="text-xs text-muted-foreground">
          {option.description}
        </span>
      </div>

      {/* Configured indicator */}
      {isConfigured && (
        <div className="absolute top-2 right-2 text-green-500">
          <CheckCircleIcon />
        </div>
      )}
    </button>
  );
}

// ============================================================================
// VALIDATION STATUS INDICATOR
// ============================================================================

interface StatusIndicatorProps {
  status: ValidationStatus;
  error: string | null;
}

function StatusIndicator({ status, error }: StatusIndicatorProps) {
  if (status === "idle") return null;

  const config = {
    validating: {
      icon: <LoadingIcon />,
      text: "Validating...",
      className: "text-muted-foreground",
    },
    valid: {
      icon: <CheckCircleIcon />,
      text: "API key is valid",
      className: "text-green-500",
    },
    invalid: {
      icon: <AlertCircleIcon />,
      text: "API key is invalid",
      className: "text-destructive",
    },
    error: {
      icon: <AlertCircleIcon />,
      text: error || "Validation failed",
      className: "text-destructive",
    },
  };

  const currentConfig = config[status];

  return (
    <div className={cn("flex items-center gap-2 text-xs", currentConfig.className)}>
      {currentConfig.icon}
      <span>{currentConfig.text}</span>
    </div>
  );
}

// ============================================================================
// AI PROVIDER CONFIG COMPONENT
// ============================================================================

/**
 * AI Provider configuration component.
 *
 * Allows users to:
 * - Select an AI provider (OpenRouter or Gemini)
 * - Enter and validate their API key
 * - Save or clear the configuration
 *
 * API keys are stored in session memory only (not persisted to disk).
 *
 * @example
 * ```tsx
 * const {
 *   aiConfig,
 *   configureAIProvider,
 *   clearAIProvider,
 *   validateAPIKey,
 *   validationStatus,
 *   validationError,
 * } = useSettings();
 *
 * <AIProviderConfig
 *   config={aiConfig}
 *   onConfigure={configureAIProvider}
 *   onClear={clearAIProvider}
 *   onValidate={validateAPIKey}
 *   validationStatus={validationStatus}
 *   validationError={validationError}
 * />
 * ```
 */
export function AIProviderConfig({
  config,
  onConfigure,
  onClear,
  onValidate,
  validationStatus,
  validationError,
  disabled = false,
  className,
}: AIProviderConfigProps) {
  // Local state for the form
  const [selectedProvider, setSelectedProvider] = useState<AIProviderType>(
    config?.provider ?? "openrouter"
  );
  const [apiKey, setApiKey] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [showKey, setShowKey] = useState(false);

  // Reset form when config changes externally
  useEffect(() => {
    if (config) {
      setSelectedProvider(config.provider);
      // Don't show the actual key, just indicate it's configured
      setApiKey("");
    }
  }, [config]);

  const isConfigured = config !== null && config.provider !== "none";
  const currentProviderOption = PROVIDER_OPTIONS.find(
    (p) => p.value === selectedProvider
  );

  // Handle provider selection
  const handleProviderSelect = useCallback((provider: AIProviderType) => {
    setSelectedProvider(provider);
    setApiKey("");
  }, []);

  // Handle validation
  const handleValidate = useCallback(async () => {
    if (!apiKey.trim()) return;
    await onValidate(selectedProvider, apiKey.trim());
  }, [selectedProvider, apiKey, onValidate]);

  // Handle save
  const handleSave = useCallback(async () => {
    if (!apiKey.trim()) return;

    setIsSubmitting(true);
    try {
      await onConfigure(selectedProvider, apiKey.trim());
      setApiKey(""); // Clear the input after saving
    } catch {
      // Error handling is done via validationError prop
    } finally {
      setIsSubmitting(false);
    }
  }, [selectedProvider, apiKey, onConfigure]);

  // Handle clear
  const handleClear = useCallback(async () => {
    setIsSubmitting(true);
    try {
      await onClear();
      setApiKey("");
    } finally {
      setIsSubmitting(false);
    }
  }, [onClear]);

  const isLoading = isSubmitting || validationStatus === "validating";
  const canSave = apiKey.trim().length > 0 && !isLoading;
  const canValidate = apiKey.trim().length > 0 && !isLoading;

  return (
    <div
      className={cn(
        "flex flex-col gap-6",
        disabled && "opacity-50 pointer-events-none",
        className
      )}
      data-slot="ai-provider-config"
    >
      {/* Current Status Banner */}
      {isConfigured && (
        <div className="flex items-center justify-between p-3 rounded-md bg-green-500/10 border border-green-500/20">
          <div className="flex items-center gap-3">
            <div className="flex items-center justify-center w-8 h-8 rounded-md bg-green-500/20 text-green-500">
              <CheckCircleIcon />
            </div>
            <div className="flex flex-col">
              <span className="text-sm font-medium">
                {PROVIDER_OPTIONS.find((p) => p.value === config?.provider)?.label} configured
              </span>
              <span className="text-xs text-muted-foreground">
                AI-guided preprocessing is available
              </span>
            </div>
          </div>
          <Button
            variant="ghost"
            size="sm"
            onClick={handleClear}
            disabled={isLoading}
            className="text-destructive hover:text-destructive hover:bg-destructive/10"
          >
            <TrashIcon />
            Remove
          </Button>
        </div>
      )}

      {/* Provider Selection */}
      <div className="flex flex-col gap-3">
        <div className="flex flex-col gap-1">
          <h3 className="text-sm font-medium">
            {isConfigured ? "Change Provider" : "Select Provider"}
          </h3>
          <p className="text-xs text-muted-foreground">
            Choose an AI provider for intelligent preprocessing decisions
          </p>
        </div>
        <div className="grid grid-cols-2 gap-3">
          {PROVIDER_OPTIONS.map((option) => (
            <ProviderCard
              key={option.value}
              option={option}
              isSelected={selectedProvider === option.value}
              isConfigured={config?.provider === option.value}
              onClick={() => handleProviderSelect(option.value)}
              disabled={disabled || isLoading}
            />
          ))}
        </div>
      </div>

      {/* API Key Input */}
      <div className="flex flex-col gap-3">
        <div className="flex items-center justify-between">
          <div className="flex flex-col gap-1">
            <h3 className="text-sm font-medium">API Key</h3>
            <p className="text-xs text-muted-foreground">
              {isConfigured && config?.provider === selectedProvider
                ? "Enter a new key to update your configuration"
                : `Enter your ${currentProviderOption?.label} API key`}
            </p>
          </div>
          {currentProviderOption && (
            <a
              href={currentProviderOption.helpUrl}
              target="_blank"
              rel="noopener noreferrer"
              className="text-xs text-primary hover:underline flex items-center"
            >
              Get API key
              <ExternalLinkIcon />
            </a>
          )}
        </div>

        <div className="flex flex-col gap-2">
          <Input
            type={showKey ? "text" : "password"}
            value={apiKey}
            onChange={(e) => setApiKey(e.target.value)}
            placeholder={currentProviderOption?.placeholder}
            disabled={disabled || isLoading}
            leftAddon={<KeyIcon />}
            rightAddon={
              <button
                type="button"
                onClick={() => setShowKey(!showKey)}
                className="text-muted-foreground hover:text-foreground transition-colors"
                tabIndex={-1}
              >
                {showKey ? <EyeOffIcon /> : <EyeIcon />}
              </button>
            }
          />

          {/* Validation Status */}
          <StatusIndicator status={validationStatus} error={validationError} />
        </div>

        {/* Action Buttons */}
        <div className="flex items-center gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={handleValidate}
            disabled={!canValidate}
          >
            {validationStatus === "validating" ? (
              <>
                <LoadingIcon />
                Validating...
              </>
            ) : (
              "Validate"
            )}
          </Button>
          <Button
            size="sm"
            onClick={handleSave}
            disabled={!canSave}
          >
            {isSubmitting ? (
              <>
                <LoadingIcon />
                Saving...
              </>
            ) : isConfigured && config?.provider === selectedProvider ? (
              "Update Key"
            ) : (
              "Save Configuration"
            )}
          </Button>
        </div>

        {/* Security Notice */}
        <p className="text-xs text-muted-foreground mt-2">
          Your API key is stored in memory only and will be cleared when you close the application.
          It is never saved to disk or sent anywhere except to the selected AI provider.
        </p>
      </div>
    </div>
  );
}

// ============================================================================
// ADDITIONAL ICONS
// ============================================================================

function EyeIcon() {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="16"
      height="16"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <path d="M2.062 12.348a1 1 0 0 1 0-.696 10.75 10.75 0 0 1 19.876 0 1 1 0 0 1 0 .696 10.75 10.75 0 0 1-19.876 0" />
      <circle cx="12" cy="12" r="3" />
    </svg>
  );
}

function EyeOffIcon() {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="16"
      height="16"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <path d="M10.733 5.076a10.744 10.744 0 0 1 11.205 6.575 1 1 0 0 1 0 .696 10.747 10.747 0 0 1-1.444 2.49" />
      <path d="M14.084 14.158a3 3 0 0 1-4.242-4.242" />
      <path d="M17.479 17.499a10.75 10.75 0 0 1-15.417-5.151 1 1 0 0 1 0-.696 10.75 10.75 0 0 1 4.446-5.143" />
      <path d="m2 2 20 20" />
    </svg>
  );
}

export default AIProviderConfig;
