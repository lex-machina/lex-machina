"use client";

import { useState, useCallback, useEffect } from "react";
import {
    Layers,
    Sparkles,
    CheckCircle2,
    AlertCircle,
    Loader2,
    KeyRound,
    Trash2,
    ExternalLink,
    Eye,
    EyeOff,
    Power,
    PowerOff,
} from "lucide-react";
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
    /** List of providers with saved API keys */
    savedProviders: AIProviderType[];
    /** Callback to configure an AI provider */
    onConfigure: (provider: AIProviderType, apiKey: string) => Promise<void>;
    /** Callback to clear the active AI provider (keeps saved keys) */
    onClear: () => Promise<void>;
    /** Callback to switch to a saved provider */
    onSwitch: (provider: AIProviderType) => Promise<void>;
    /** Callback to permanently delete a saved provider's key */
    onDelete: (provider: AIProviderType) => Promise<void>;
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
        icon: <Layers className="h-5 w-5 shrink-0" />,
    },
    {
        value: "gemini",
        label: "Google Gemini",
        description: "Google's advanced AI model",
        placeholder: "AIza...",
        helpUrl: "https://aistudio.google.com/apikey",
        icon: <Sparkles className="h-5 w-5 shrink-0" />,
    },
];

// ============================================================================
// PROVIDER CARD COMPONENT
// ============================================================================

interface ProviderCardProps {
    option: ProviderOption;
    isActive: boolean;
    hasSavedKey: boolean;
    onActivate: () => void;
    onDeactivate: () => void;
    onDelete: () => void;
    onSelect: () => void;
    isSelected: boolean;
    disabled?: boolean;
    isLoading?: boolean;
}

function ProviderCard({
    option,
    isActive,
    hasSavedKey,
    onActivate,
    onDeactivate,
    onDelete,
    onSelect,
    isSelected,
    disabled,
    isLoading,
}: ProviderCardProps) {
    return (
        <div
            className={cn(
                "relative flex flex-col gap-3 rounded-md p-3",
                "border transition-all duration-150",
                isSelected ? "border-primary bg-primary/5" : "border-border",
                disabled && "pointer-events-none opacity-50",
            )}
        >
            {/* Header */}
            <button
                type="button"
                onClick={onSelect}
                disabled={disabled}
                className={cn(
                    "flex w-full items-center gap-3 text-left",
                    "focus-visible:ring-ring focus:outline-none focus-visible:ring-2 focus-visible:ring-offset-2",
                )}
            >
                {/* Icon */}
                <div
                    className={cn(
                        "flex h-10 w-10 items-center justify-center rounded-md",
                        isActive
                            ? "bg-primary text-primary-foreground"
                            : hasSavedKey
                              ? "bg-muted text-foreground"
                              : "bg-muted text-muted-foreground",
                    )}
                >
                    {option.icon}
                </div>

                {/* Content */}
                <div className="flex min-w-0 flex-1 flex-col gap-0.5">
                    <span className="text-sm font-medium">{option.label}</span>
                    <span className="text-muted-foreground text-xs">
                        {option.description}
                    </span>
                </div>
            </button>

            {/* Status & Actions */}
            <div className="flex items-center justify-between gap-2 pl-[52px]">
                {/* Status indicators */}
                <div className="flex items-center gap-2">
                    {isActive && (
                        <span className="text-primary flex items-center gap-1 text-xs">
                            <CheckCircle2 className="h-3 w-3" />
                            Active
                        </span>
                    )}
                    {!isActive && hasSavedKey && (
                        <span className="text-muted-foreground flex items-center gap-1 text-xs">
                            <KeyRound className="h-3 w-3" />
                            Key saved
                        </span>
                    )}
                </div>

                {/* Action buttons */}
                <div className="flex items-center gap-1">
                    {isActive ? (
                        <Button
                            variant="ghost"
                            size="sm"
                            onClick={onDeactivate}
                            disabled={isLoading}
                            className="text-muted-foreground hover:text-foreground h-7 px-2 text-xs"
                        >
                            <PowerOff className="mr-1 h-3 w-3" />
                            Deactivate
                        </Button>
                    ) : hasSavedKey ? (
                        <Button
                            variant="ghost"
                            size="sm"
                            onClick={onActivate}
                            disabled={isLoading}
                            className="text-primary hover:text-primary h-7 px-2 text-xs"
                        >
                            <Power className="mr-1 h-3 w-3" />
                            Activate
                        </Button>
                    ) : null}
                    {hasSavedKey && (
                        <Button
                            variant="ghost"
                            size="sm"
                            onClick={onDelete}
                            disabled={isLoading}
                            className="text-destructive hover:text-destructive hover:bg-destructive/10 h-7 px-2 text-xs"
                        >
                            <Trash2 className="h-3 w-3" />
                        </Button>
                    )}
                </div>
            </div>
        </div>
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
            icon: <Loader2 className="h-4 w-4 shrink-0 animate-spin" />,
            text: "Validating...",
            className: "text-muted-foreground",
        },
        valid: {
            icon: <CheckCircle2 className="h-4 w-4 shrink-0" />,
            text: "API key is valid",
            className: "text-green-500",
        },
        invalid: {
            icon: <AlertCircle className="h-4 w-4 shrink-0" />,
            text: "API key is invalid",
            className: "text-destructive",
        },
        error: {
            icon: <AlertCircle className="h-4 w-4 shrink-0" />,
            text: error || "Validation failed",
            className: "text-destructive",
        },
    };

    const currentConfig = config[status];

    return (
        <div
            className={cn(
                "flex items-center gap-2 text-xs",
                currentConfig.className,
            )}
        >
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
 * - Save multiple provider keys
 * - Switch between saved providers
 * - Delete saved provider keys
 *
 * API keys are stored securely in the OS keychain.
 */
export function AIProviderConfig({
    config,
    savedProviders,
    onConfigure,
    onClear,
    onSwitch,
    onDelete,
    onValidate,
    validationStatus,
    validationError,
    disabled = false,
    className,
}: AIProviderConfigProps) {
    // Local state for the form
    const [selectedProvider, setSelectedProvider] = useState<AIProviderType>(
        config?.provider ?? "openrouter",
    );
    const [apiKey, setApiKey] = useState("");
    const [isSubmitting, setIsSubmitting] = useState(false);
    const [showKey, setShowKey] = useState(false);

    // Reset form when config changes externally
    useEffect(() => {
        if (config) {
            setSelectedProvider(config.provider);
            setApiKey("");
        }
    }, [config]);

    const currentProviderOption = PROVIDER_OPTIONS.find(
        (p) => p.value === selectedProvider,
    );

    // Handle provider selection
    const handleProviderSelect = useCallback((provider: AIProviderType) => {
        setSelectedProvider(provider);
        setApiKey("");
    }, []);

    // Handle activation (switch to saved provider)
    const handleActivate = useCallback(
        async (provider: AIProviderType) => {
            setIsSubmitting(true);
            try {
                await onSwitch(provider);
            } finally {
                setIsSubmitting(false);
            }
        },
        [onSwitch],
    );

    // Handle deactivation (clear active provider, keep saved key)
    const handleDeactivate = useCallback(async () => {
        setIsSubmitting(true);
        try {
            await onClear();
        } finally {
            setIsSubmitting(false);
        }
    }, [onClear]);

    // Handle deletion (permanently remove saved key)
    const handleDelete = useCallback(
        async (provider: AIProviderType) => {
            setIsSubmitting(true);
            try {
                await onDelete(provider);
            } finally {
                setIsSubmitting(false);
            }
        },
        [onDelete],
    );

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

    const isLoading = isSubmitting || validationStatus === "validating";
    const canSave = apiKey.trim().length > 0 && !isLoading;
    const canValidate = apiKey.trim().length > 0 && !isLoading;
    const selectedHasSavedKey = savedProviders.includes(selectedProvider);

    return (
        <div
            className={cn(
                "flex flex-col gap-6",
                disabled && "pointer-events-none opacity-50",
                className,
            )}
            data-slot="ai-provider-config"
        >
            {/* Provider Selection */}
            <div className="flex flex-col gap-3">
                <div className="flex flex-col gap-1">
                    <h3 className="text-sm font-medium">AI Providers</h3>
                    <p className="text-muted-foreground text-xs">
                        Configure AI providers for intelligent preprocessing
                        decisions. You can save keys for multiple providers and
                        switch between them.
                    </p>
                </div>
                <div className="flex flex-col gap-2">
                    {PROVIDER_OPTIONS.map((option) => (
                        <ProviderCard
                            key={option.value}
                            option={option}
                            isActive={config?.provider === option.value}
                            hasSavedKey={savedProviders.includes(option.value)}
                            onActivate={() => handleActivate(option.value)}
                            onDeactivate={handleDeactivate}
                            onDelete={() => handleDelete(option.value)}
                            onSelect={() => handleProviderSelect(option.value)}
                            isSelected={selectedProvider === option.value}
                            disabled={disabled}
                            isLoading={isLoading}
                        />
                    ))}
                </div>
            </div>

            {/* API Key Input */}
            <div className="flex flex-col gap-3">
                <div className="flex items-center justify-between">
                    <div className="flex flex-col gap-1">
                        <h3 className="text-sm font-medium">
                            {selectedHasSavedKey
                                ? "Update API Key"
                                : "Add API Key"}
                        </h3>
                        <p className="text-muted-foreground text-xs">
                            {selectedHasSavedKey
                                ? `Enter a new key to update your ${currentProviderOption?.label} configuration`
                                : `Enter your ${currentProviderOption?.label} API key to enable AI-guided preprocessing`}
                        </p>
                    </div>
                    {currentProviderOption && (
                        <a
                            href={currentProviderOption.helpUrl}
                            target="_blank"
                            rel="noopener noreferrer"
                            className="text-primary flex items-center text-xs hover:underline"
                        >
                            Get API key
                            <ExternalLink className="ml-1 h-3 w-3 shrink-0" />
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
                        leftAddon={<KeyRound className="h-4 w-4 shrink-0" />}
                        rightAddon={
                            <button
                                type="button"
                                onClick={() => setShowKey(!showKey)}
                                className="text-muted-foreground hover:text-foreground transition-colors"
                                tabIndex={-1}
                            >
                                {showKey ? (
                                    <EyeOff className="h-4 w-4" />
                                ) : (
                                    <Eye className="h-4 w-4" />
                                )}
                            </button>
                        }
                    />

                    {/* Validation Status */}
                    <StatusIndicator
                        status={validationStatus}
                        error={validationError}
                    />
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
                                <Loader2 className="h-4 w-4 shrink-0 animate-spin" />
                                Validating...
                            </>
                        ) : (
                            "Validate"
                        )}
                    </Button>
                    <Button size="sm" onClick={handleSave} disabled={!canSave}>
                        {isSubmitting ? (
                            <>
                                <Loader2 className="h-4 w-4 shrink-0 animate-spin" />
                                Saving...
                            </>
                        ) : selectedHasSavedKey ? (
                            "Update Key"
                        ) : (
                            "Save Key"
                        )}
                    </Button>
                </div>

                {/* Security Notice */}
                <p className="text-muted-foreground mt-2 text-xs">
                    Your API keys are stored securely in your operating
                    system&apos;s keychain and persist across app restarts. They
                    are never sent anywhere except to the selected AI provider.
                </p>
            </div>
        </div>
    );
}

export default AIProviderConfig;
