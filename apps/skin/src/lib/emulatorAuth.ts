import {
  auth,
  createUserWithEmailAndPassword,
  signInWithEmailAndPassword,
} from "./firebase";

export const QA_EMAIL = "qa@terrarium.dev";
export const QA_PASSWORD = "qa-terrarium";

/** Sign in (or create) the local emulator QA user — no Google popup. */
export async function signInWithEmulatorQaUser() {
  try {
    await signInWithEmailAndPassword(auth, QA_EMAIL, QA_PASSWORD);
  } catch (err) {
    const code = err instanceof Error && "code" in err ? String((err as { code: string }).code) : "";
    if (code === "auth/user-not-found" || code === "auth/invalid-credential") {
      await createUserWithEmailAndPassword(auth, QA_EMAIL, QA_PASSWORD);
      return;
    }
    throw err;
  }
}
