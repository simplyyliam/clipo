import React from "react";
import { createBrowserRouter, type RouteObject } from "react-router-dom";
import AppLayout from "@/views/app/layout/AppLayout";

// Dynamically discover all view pages using Vite's import.meta.glob
const pageModules = import.meta.glob<{ default: React.ComponentType }>(
  "./views/**/page.tsx",
  { eager: true }
);

function buildRoutes(): RouteObject[] {
  const appChildren: RouteObject[] = [];
  const rootRoutes: RouteObject[] = [];

  for (const path in pageModules) {
    const Component = pageModules[path].default;
    if (!Component) continue;

    // Convert ./views/app/settings/page.tsx -> /settings
    // Convert ./views/app/page.tsx -> /
    // Convert ./views/login/page.tsx -> /login (standalone view)
    const normalized = path
      .replace("./views", "")
      .replace("/page.tsx", "")
      .replace(/\/index$/, "");

    if (normalized.startsWith("/app")) {
      const childPath = normalized.replace("/app", "").replace(/^\//, "");
      appChildren.push({
        path: childPath === "" ? undefined : childPath,
        index: childPath === "",
        element: <Component />,
      });
    } else {
      const routePath = normalized === "" ? "/" : normalized;
      rootRoutes.push({
        path: routePath,
        element: <Component />,
      });
    }
  }

  return [
    {
      path: "/",
      element: <AppLayout />,
      children: appChildren,
    },
    ...rootRoutes,
  ];
}

export const Router = createBrowserRouter(buildRoutes())