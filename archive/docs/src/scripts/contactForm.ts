import Swal from 'sweetalert2';
import { db } from '../lib/firebase';
import { collection, addDoc, serverTimestamp } from 'firebase/firestore';

export function setupContactForm() {
  const form = document.getElementById("contact-form") as HTMLFormElement | null;
  const status = document.getElementById("form-status");
  const sendButton = form?.querySelector('button[type="submit"]') as HTMLButtonElement | null;

  if (!form || !status || !sendButton) return;

  form.addEventListener("submit", async (e) => {
    e.preventDefault();

    const nameInput = form.querySelector("#name") as HTMLInputElement;
    const emailInput = form.querySelector("#email") as HTMLInputElement;
    const messageInput = form.querySelector("#message") as HTMLTextAreaElement;

    const name = nameInput?.value.trim();
    const email = emailInput?.value.trim();
    const message = messageInput?.value.trim();
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

    // Client-side validation
    if (!name || !email || !message) {
      Swal.fire({
        icon: "warning",
        title: "Please fill in all fields",
        toast: true,
        position: 'top-end',
        timer: 3000,
        timerProgressBar: true,
        showConfirmButton: false,
      });
      return;
    }

    if (!emailRegex.test(email)) {
      Swal.fire({
        icon: "error",
        title: "Please enter a valid email",
        toast: true,
        position: 'top-end',
        timer: 3000,
        timerProgressBar: true,
        showConfirmButton: false,
      });
      return;
    }

    const recaptchaToken = (window as any).grecaptcha?.getResponse();
    if (!recaptchaToken) {
      Swal.fire({
        icon: "warning",
        title: "Please complete the reCAPTCHA",
        toast: true,
        position: 'top-end',
        timer: 3000,
        timerProgressBar: true,
        showConfirmButton: false,
      });
      return;
    }

    // Verify reCAPTCHA with backend
    try {
      const verifyResponse = await fetch("/api/verify-recaptcha", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ token: recaptchaToken }),
      });

      const verifyResult = await verifyResponse.json();
      if (!verifyResult.success) {
        Swal.fire({
          icon: "error",
          title: "reCAPTCHA verification failed",
          text: "Please try again later.",
          toast: true,
          position: 'top-end',
          timer: 3000,
          timerProgressBar: true,
          showConfirmButton: false,
        });
        return;
      }
    } catch (error) {
      console.error("reCAPTCHA verification error:", error);
      Swal.fire({
        icon: "error",
        title: "Could not verify reCAPTCHA",
        text: "Network or server issue.",
        toast: true,
        position: 'top-end',
        timer: 3000,
        timerProgressBar: true,
        showConfirmButton: false,
      });
      return;
    }

    // Proceed to send
    sendButton.disabled = true;
    status.textContent = "Sending...";

    try {
      await addDoc(collection(db, "messages"), {
        name,
        email,
        message,
        createdAt: serverTimestamp()
      });

      status.textContent = "Message sent!";
      form.reset();
      (window as any).grecaptcha?.reset();

      Swal.fire({
        toast: true,
        position: 'top-end',
        icon: 'success',
        title: 'Message sent successfully',
        showConfirmButton: false,
        timer: 3000,
        timerProgressBar: true,
      });
    } catch (err) {
      console.error(err);
      status.textContent = "Something went wrong. Try again later.";

      Swal.fire({
        toast: true,
        position: 'top-end',
        icon: 'error',
        title: 'Failed to send message',
        showConfirmButton: false,
        timer: 3000,
        timerProgressBar: true,
      });
    } finally {
      sendButton.disabled = false;
    }
  });
}
