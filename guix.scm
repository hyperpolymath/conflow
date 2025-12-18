;; SPDX-License-Identifier: MIT OR AGPL-3.0-or-later
;; SPDX-FileCopyrightText: 2024-2025 hyperpolymath
;; conflow - Guix Package Definition
;; Run: guix shell -D -f guix.scm

(use-modules (guix packages)
             (guix gexp)
             (guix git-download)
             (guix build-system cargo)
             ((guix licenses) #:prefix license:)
             (gnu packages base))

(define-public conflow
  (package
    (name "conflow")
    (version "0.1.0")
    (source (local-file "." "conflow-checkout"
                        #:recursive? #t
                        #:select? (git-predicate ".")))
    (build-system cargo-build-system)
    (synopsis "Configuration Flow Orchestrator")
    (description "Intelligently orchestrate CUE, Nickel, and configuration validation workflows.")
    (home-page "https://gitlab.com/hyperpolymath/conflow")
    (license (list license:expat license:agpl3+))))

;; Return package for guix shell
conflow
