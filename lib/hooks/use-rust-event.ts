"use client";

import { useEffect, useRef } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

/**
 * Generic hook for subscribing to Tauri events from the Rust backend.
 *
 * This hook handles the lifecycle of event subscriptions automatically:
 * - Subscribes when the component mounts
 * - Unsubscribes when the component unmounts
 * - Re-subscribes if the event name changes
 *
 * @template T - The type of the event payload
 * @param eventName - The name of the event to listen for (e.g., "file:loaded")
 * @param handler - Callback function invoked when the event is received
 *
 * @example
 * ```tsx
 * // Listen for file loaded events
 * useRustEvent<FileLoadedPayload>("file:loaded", (payload) => {
 *   console.log("File loaded:", payload.file_info.name);
 * });
 *
 * // Listen for loading state changes
 * useRustEvent<LoadingPayload>("app:loading", (payload) => {
 *   setIsLoading(payload.is_loading);
 * });
 * ```
 *
 * @remarks
 * - The handler is stored in a ref to avoid unnecessary re-subscriptions
 * - If you need to react to state changes in the handler, ensure those
 *   dependencies are handled appropriately (the handler ref is updated on each render)
 */
export function useRustEvent<T>(
    eventName: string,
    handler: (payload: T) => void,
): void {
    // Store handler in ref to avoid re-subscribing when handler changes
    const handlerRef = useRef(handler);

    // Update ref on each render to ensure we always call the latest handler
    useEffect(() => {
        handlerRef.current = handler;
    });

    useEffect(() => {
        let unlisten: UnlistenFn | undefined;
        let mounted = true;

        // Subscribe to the event
        const subscribe = async () => {
            try {
                unlisten = await listen<T>(eventName, (event) => {
                    if (mounted) {
                        handlerRef.current(event.payload);
                    }
                });
            } catch (error) {
                console.error(
                    `Failed to subscribe to event "${eventName}":`,
                    error,
                );
            }
        };

        subscribe();

        // Cleanup: unsubscribe when component unmounts or eventName changes
        return () => {
            mounted = false;
            if (unlisten) {
                unlisten();
            }
        };
    }, [eventName]);
}

/**
 * Hook for subscribing to multiple Tauri events at once.
 *
 * Useful when you need to listen to several related events in a single component.
 *
 * @param subscriptions - Array of event subscriptions with name and handler
 *
 * @example
 * ```tsx
 * useRustEvents([
 *   { name: "file:loaded", handler: (p) => setFile(p.file_info) },
 *   { name: "file:closed", handler: () => setFile(null) },
 * ]);
 * ```
 */
export function useRustEvents<T extends Record<string, unknown>>(
    subscriptions: Array<{
        name: string;
        handler: (payload: T[keyof T]) => void;
    }>,
): void {
    const handlersRef = useRef(subscriptions);

    useEffect(() => {
        handlersRef.current = subscriptions;
    });

    useEffect(() => {
        const unlisteners: UnlistenFn[] = [];
        let mounted = true;

        const subscribe = async () => {
            for (const { name } of subscriptions) {
                try {
                    const unlisten = await listen(name, (event) => {
                        if (mounted) {
                            // Find the current handler for this event name
                            const subscription = handlersRef.current.find(
                                (s) => s.name === name,
                            );
                            if (subscription) {
                                subscription.handler(
                                    event.payload as T[keyof T],
                                );
                            }
                        }
                    });
                    unlisteners.push(unlisten);
                } catch (error) {
                    console.error(
                        `Failed to subscribe to event "${name}":`,
                        error,
                    );
                }
            }
        };

        subscribe();

        return () => {
            mounted = false;
            unlisteners.forEach((unlisten) => unlisten());
        };
        // Re-subscribe if the subscription names change
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [subscriptions.map((s) => s.name).join(",")]);
}
