"use client";

import { useState, useEffect } from "react";

interface Toast {
    id: string;
    message: string;
    type: "success" | "error" | "info";
}

let toastId = 0;

const listeners: Set<(toasts: Toast[]) => void> = new Set();

let toasts: Toast[] = [];

const notifyListeners = () => {
    listeners.forEach((listener) => listener([...toasts]));
};

export const toast = {
    dismiss: (id: string) => {
        toasts = toasts.filter((t) => t.id !== id);
        notifyListeners();
    },

    success: (message: string) => {
        const id = `toast-${++toastId}`;
        toasts = [...toasts, { id, message, type: "success" }];
        notifyListeners();
        setTimeout(() => toast.dismiss(id), 5000);
    },

    error: (message: string) => {
        const id = `toast-${++toastId}`;
        toasts = [...toasts, { id, message, type: "error" }];
        notifyListeners();
        setTimeout(() => toast.dismiss(id), 5000);
    },

    info: (message: string) => {
        const id = `toast-${++toastId}`;
        toasts = [...toasts, { id, message, type: "info" }];
        notifyListeners();
        setTimeout(() => toast.dismiss(id), 5000);
    },
};

const useToast = () => {
    const [state, setState] = useState<Toast[]>([]);

    useEffect(() => {
        listeners.add(setState);
        return () => {
            listeners.delete(setState);
        };
    }, []);

    return state;
};

const ToastContainer = () => {
    const toastList = useToast();

    if (toastList.length === 0) {
        return null;
    }

    return (
        <div className="fixed right-5 bottom-5 z-50 flex flex-col gap-2">
            {toastList.map((t) => (
                <div
                    key={t.id}
                    className={`rounded-md px-4 py-3 text-sm font-medium shadow-lg transition-all ${
                        t.type === "success"
                            ? "bg-green-600 text-white"
                            : t.type === "error"
                              ? "bg-red-600 text-white"
                              : "bg-blue-600 text-white"
                    }`}
                >
                    <div className="flex items-center gap-2">
                        <span>{t.message}</span>
                        <button
                            onClick={() => {
                                toast.dismiss(t.id);
                            }}
                            className="ml-2 opacity-70 hover:opacity-100"
                        >
                            x
                        </button>
                    </div>
                </div>
            ))}
        </div>
    );
};

export default ToastContainer;
