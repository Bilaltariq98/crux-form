# Survey Form Application

A modern survey form application built with Rust (backend) and Next.js (frontend), using Crux for shared business logic.

## Project Structure

```
.
├── frontend/          # Next.js frontend application
├── shared/           # Shared business logic (Rust)
└── shared_types/     # Shared type definitions
```

## Prerequisites

- Rust (latest stable version)
- Node.js (v18 or later)
- pnpm (latest version)

## Build Instructions

1. First, build the shared libraries:
   ```bash
   # Build the shared library
   cd shared
   cargo build
   
   # Build the shared types
   cd ../shared_types
   cargo build
   ```

2. After building the shared libraries, install frontend dependencies:
   ```bash
   cd frontend
   pnpm install
   ```

3. Start the development server:
   ```bash
   pnpm dev
   ```

## Important Notes

- The shared libraries (`shared/` and `shared_types/`) **must** be built before running `pnpm install` in the frontend directory
- The application uses Crux for shared business logic between Rust and TypeScript
- The frontend is built with Next.js and uses Tailwind CSS for styling

## Development

- Frontend runs on `http://localhost:3000` by default
- The application implements a form with validation, editing capabilities, and state management
- All business logic is shared between Rust and TypeScript through Crux

## License

MIT 