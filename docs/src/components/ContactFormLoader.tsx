// src/components/ContactFormLoader.tsx
import { useEffect } from "react";
import { setupContactForm } from "../scripts/contactForm";

export default function ContactFormLoader() {
  useEffect(() => {
    setupContactForm();
  }, []);

  return null;
}
