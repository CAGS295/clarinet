import * as vscode from "vscode";
import { Uri } from "vscode";

import { LanguageClient } from "./types";
import { fileArrayToString } from "./utils/files";

const { fs } = vscode.workspace;

function isValidReadEvent(e: any): e is { path: string } {
  return typeof e?.path === "string";
}

function isValidReadManyEvent(e: any): e is { paths: string[] } {
  return (
    Array.isArray(e?.paths) &&
    e.paths.every((s: unknown) => typeof s === "string")
  );
}

function isValidWriteEvent(e: any): e is { path: string; content: number[] } {
  return typeof e?.path === "string" && Array.isArray(e?.content);
}

export function initVFS(client: LanguageClient) {
  client.onRequest("vfs/exists", async (event: unknown) => {
    if (!isValidReadEvent(event)) throw new Error("invalid read event");
    try {
      await fs.stat(Uri.parse(event.path));
      return true;
    } catch {
      return false;
    }
  });

  client.onRequest("vfs/readFile", async (event: unknown) => {
    if (!isValidReadEvent(event)) throw new Error("invalid read event");
    return fileArrayToString(await fs.readFile(Uri.parse(event.path)));
  });

  client.onRequest("vfs/readFiles", async (event: any) => {
    if (!isValidReadManyEvent(event)) throw new Error("invalid read event");
    const files = await Promise.all(
      event.paths.map((p) => fs.readFile(Uri.parse(p))),
    );
    return Object.fromEntries(
      files.map((f, i) => [event.paths[i], fileArrayToString(f)]),
    );
  });

  client.onRequest("vfs/writeFile", async (event: unknown) => {
    if (!isValidWriteEvent(event)) throw new Error("invalid write event");
    return fs.writeFile(Uri.parse(event.path), Uint8Array.from(event.content));
  });
}
