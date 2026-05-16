---
description: "Guide and complete the Shudong Agent project - analyze code structure, generate scaffolding, create tests/docs, and deliver the final product combining Flutter UI and Rust core"
name: "Project Completion Guide"
agent: "agent"
---

# Shudong Agent Project Completion Guide

You are helping complete a Flutter + Rust hybrid project (Shudong Agent). Analyze the current state and provide comprehensive guidance to take it from early stage to a complete, production-ready product.

## Step 1: Project Analysis
First, examine the current structure:
- **Flutter App** (`app/`): UI layer, Dart code in `lib/`, platform configurations (macOS, Linux, Windows)
- **Rust Core** (`core/`): Backend logic using Cargo, located in `core/src/`
- **Current Status**: Early stage project setup

Identify:
1. What's already implemented and working
2. What's missing or incomplete
3. Architecture issues or opportunities for improvement
4. Integration points between Dart and Rust

## Step 2: Project Structure Assessment
Evaluate and suggest improvements for:
- Directory organization and naming conventions
- Separation of concerns (UI, business logic, data layer)
- Configuration management (pubspec.yaml, Cargo.toml)
- Build system setup and cross-platform compatibility

## Step 3: Code Generation & Scaffolding
Based on the analysis, provide:
1. **Missing files** needed for a complete project
2. **Skeleton code** for unimplemented features
3. **Recommended patterns** for Dart + Rust integration
4. **Configuration templates** (if needed for build/deployment)

## Step 4: Testing & Documentation
Suggest:
1. **Test structure**: Unit tests for Dart and Rust, integration tests
2. **Test templates**: Example test cases based on the project's functionality
3. **Documentation**: README structure, API documentation, setup instructions
4. **Code comments**: Areas that need documentation

## Step 5: Integration Points
Provide specific guidance on:
1. How to call Rust code from Dart (FFI or bindings)
2. Data serialization between layers
3. Error handling across boundaries
4. Platform-specific considerations

## Step 6: Completion Roadmap
Create an actionable plan with:
1. Priority order for completing features
2. Estimated effort for each phase
3. Dependencies between tasks
4. Milestones and checkpoints

## Output Format
Provide:
- Clear analysis with specific findings
- Concrete code examples where helpful
- Direct file paths for improvements
- Step-by-step completion checklist
- Risk assessment and mitigation strategies

**Ready to help complete your project. Begin with a thorough analysis and provide actionable next steps.**
