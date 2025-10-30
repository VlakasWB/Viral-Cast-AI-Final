# Product Specification — Viral Cast AI Frontend

Version: 1.0  
Date: 2025-10-19  
Status: Draft

## 1. Summary
Viral Cast AI Frontend is a SvelteKit application for store management, content, recipes, and AI/automation integration. The feature focus includes authentication, profile and settings management, product catalog, ingredients, units of measurement (UOM), recipes, orders, BMKG weather, and a modern responsive UI/UX.

## 2. Purpose
- Enable ops and content teams to manage inventory, recipes, orders, and promotions.
- Provide a fast, secure, consistent interface across desktop and mobile.
- Establish a foundation for future automation and AI-driven content/recommendations.

## 3. User Personas
- Admin: manages configuration, users, access rights, and master data.
- Editor/Content: manages recipes, ingredients, products, and content publication.
- Operations/Store: handles orders, stock, and operational weather information.

## 4. Scope
- SvelteKit frontend connected to the Backend API (`API_BASE_URL`), default: `http://localhost:12000`.
- Local production preview available at `http://localhost:4173` for QA/UI testing.

## 5. Key Features
- Authentication
  - Login, logout; validation via `/api/v1/auth/login` with token-based sessions.
- Profile & Settings
  - “Settings” page where fields are initially non-editable; become editable after clicking “Edit”.
- Store Management
  - Store list, details, open hours configuration, and active/inactive status.
- Products
  - Product CRUD, image upload, categorization, and pricing.
- Ingredients
  - Ingredient CRUD, stock, suppliers, and recipe associations.
- UOM (Units of Measurement)
  - UOM CRUD, conversions, and integration with ingredients/products.
- Recipes
  - Recipe CRUD, steps, linked ingredients, portions, and costing.
- Orders
  - Order list, status, item details, and operational actions.
- BMKG Weather
  - Region selection down to village/subdistrict; fetch operational weather forecasts.
- UI/UX
  - Sidebar navigation, responsive header, light/dark mode, baseline accessibility.

## 6. Core Functional Flows
- Login
  1. User navigates to `/login`.
  2. Enter credentials and submit to the backend.
  3. Store token/session, redirect to home or `from` URL if provided.
- Settings
  1. Fields are non-editable on initial load.
  2. Click “Edit” to enable inputs; Save sends PATCH/PUT to the API.
- Products
  1. Add product: name, category, price, description, image (preview).
  2. Save; appears in the list; can be edited/deleted.
- Recipes
  1. Create recipe, select ingredients (linked to ingredients), set portions & steps.
  2. Save; shown in the list; integrate costing/calculation.
- Orders
  1. View list; open item details.
  2. Actions: change status, add operational notes.
- Weather
  1. Select province → regency/city → district → village/subdistrict.
  2. Display forecast from integrated BMKG API.

## 7. Technical Requirements
- Platform: SvelteKit, Node.js 20+, pnpm.
- Build/Preview:
  - `pnpm run build` for production builds.
  - `pnpm run preview -- --port 4173` for local preview.
- Environment:
  - `API_BASE_URL` default `http://localhost:12000`.
  - `FRONTEND_URL` adjusted (e.g., `http://localhost:4173` during QA).
- Security:
  - Token-based auth, route protection, input sanitization, and backend-aligned CORS.
- Performance:
  - Lazy-loaded components, light caching, and minimal re-renders.

## 8. API Integration (Examples)
- Auth: `POST /api/v1/auth/login` → token/session.
- UOM: `GET/POST/PATCH /api/v1/uoms` and `GET/PUT/PATCH /api/v1/uoms/{uuid}`.
- BMKG Weather: `GET /api/v1/weather_bmkg/prediction?region_code=...`.
- Recipes/Products/Ingredients/Orders: CRUD endpoints per backend schema.

## 9. Testing & Validation
- Manual QA
  - Login using `admin_vcai_2` / `admin` at `http://localhost:4173/login`.
  - Open Settings at `http://localhost:4173/settings`: verify non-edit state; click “Edit” to make editable; save changes.
- Automated (plan)
  - Playwright E2E (login, navigation, Settings toggle).
  - Reports sent to TestSprite MCP once stabilized.
- Unit & Integration
  - Unit tests for utils/stores; integration tests for API services and key pages.

## 10. Error Handling
- Show concise, actionable error messages without exposing sensitive details.
- Client-side logging for debugging (non-PII), fallback UI when API is down.

## 11. Success Criteria
- Login functions correctly with proper redirects.
- Settings page behavior: non-edit → click “Edit” → editable → save.
- Core CRUD (products, ingredients, UOM, recipes) works without critical errors.
- BMKG weather preview displays correctly based on region selection.

## 12. Release Plan
- Local QA (preview 4173).
- UX refinements & API stabilization.
- Automated report integration to TestSprite MCP.

## 13. Implementation Notes
- `.dockerignore` excludes `.md` from container build context—OK for repo documentation.
- Use `pnpm dlx md-to-pdf` for local PDF conversion; Puppeteer will download Chromium if needed.

---
This document is placed at `testsprite_tests/Product_Spec_Viral_Cast_AI.en.md` for easy retrieval by the TestSprite MCP pipeline.