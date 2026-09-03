import {
  auth,
  createUserWithEmailAndPassword,
  signInWithEmailAndPassword,
} from "./firebase";

export const E2E_USER_EMAIL = "qa@terrarium.dev";
export const E2E_USER_PASSWORD = "qa-terrarium";

/** Sign in (or create) the local emulator test user — no Google popup. */
export async function signInWithEmulatorTestUser() {
  try {
    await signInWithEmailAndPassword(auth, E2E_USER_EMAIL, E2E_USER_PASSWORD);
  } catch (err) {
    const code = err instanceof Error && "code" in err ? String((err as { code: string }).code) : "";
    if (code === "auth/user-not-found" || code === "auth/invalid-credential") {
      await createUserWithEmailAndPassword(auth, E2E_USER_EMAIL, E2E_USER_PASSWORD);
      return;
    }
    throw err;
  }
}
