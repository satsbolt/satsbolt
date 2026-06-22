# SatsBolt Mobile Client ⚡

The native cross-platform mobile client for SatsBolt, built with Flutter.

This application features two user modes:
1. **Social Creator Mode**: Instant on-chain and off-chain (zero-fee) tipping by username or scanning QR codes.
2. **Merchant Business Mode**: Point of Sale interfaces for generating dynamic QR invoice codes, transaction histories, and basic volume metrics.

---

## 1. Prerequisites & Stack

Ensure you have the following installed locally:
- **Flutter SDK** (Version 3.10+ recommended)
- **Dart SDK** (Version 3.0+)
- **Android Studio / Xcode** (for emulators and SDK tooling)
- **CocoaPods** (for iOS builds)
- **Just Runner** (for executing tasks in the workspace)

---

## 2. Directory Structure

The application code resides in `lib/` and follows a structured architecture using **GetX** for state management:

```
lib/
├── bindings/       # GetX dependency injections (attaches controllers to views)
├── controllers/    # Business logic, state tracking, and API integrations
├── models/         # JSON serializable data models
├── services/       # Persistent services (auth storage, API clients)
├── views/          # Screen UI components and widgets
│   ├── auth/       # Login, Register, Profile screens
│   ├── creator/    # Tipping views, leaderboards, balance display
│   └── merchant/   # Invoicing, QR code generation, transaction history
└── main.dart       # App initialization entry point
```

---

## 3. Local Development Setup

### Step 1: Initialize Dependencies
Fetch Dart/Flutter dependencies:
```bash
just get-frontend
```

### Step 2: Configure Environment
Specify the backend API URL. In local development:
- **Android Emulator**: Set base API url to `http://10.0.2.2:8080` (loopback to host).
- **iOS Simulator / Desktop**: Set base API url to `http://localhost:8080`.
- **Physical Device**: Use your local network IP (e.g. `http://192.168.1.X:8080`).

Ensure this matches the `PORT` and `HOST` configured in your root backend `.env` file.

### Step 3: Run the Application
Start the development server/application:
```bash
flutter run
```
To run on a specific device, list available devices via `flutter devices` and execute:
```bash
flutter run -d <device-id>
```

---

## 4. Verification & Testing

Before submitting pull requests, run Dart analyzer checks and the widget testing suite:

```bash
# Check formatting
just fmt-frontend

# Run analysis linter
just lint-frontend

# Run unit & widget tests
just test-frontend
```
