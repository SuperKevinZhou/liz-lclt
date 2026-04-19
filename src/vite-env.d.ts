/// <reference types="vite/client" />

declare module "@tauri-apps/plugin-dialog" {
  export interface ConfirmDialogOptions {
    title?: string;
    kind?: "info" | "warning" | "error";
    okLabel?: string;
    cancelLabel?: string;
  }

  export interface OpenDialogOptions {
    title?: string;
    multiple?: boolean;
    directory?: boolean;
    recursive?: boolean;
    defaultPath?: string;
  }

  export function confirm(
    message: string,
    options?: string | ConfirmDialogOptions,
  ): Promise<boolean>;

  export function open<T extends OpenDialogOptions>(
    options?: T,
  ): Promise<string | string[] | null>;
}
