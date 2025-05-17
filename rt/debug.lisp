;; non-standard utilities to help with debugging programs (and the compiler itself)

(defun assert (that-not-nil or-else-panic-msg)
    (if that-not-nil nil (panic or-else-panic-msg)))

(defun panic (message)
    (intrinsic:princ message)
    (intrinsic:panic))

(defun to-string-list-items (thingy)
    (if (cdr thingy)
        (concatenate 'string (to-string-any (car thingy)) " " (to-string-list-items (cdr thingy)))
        (to-string-any (car thingy))))

(defun to-string-list (thingy)
    (concatenate 'string "(" (to-string-list-items thingy) ")"))

(defun to-string-number (thingy)
    (if (< thingy 0)
        (concatenate 'string "-" (to-string-number (- thingy)))
        (if (< thingy 10)
            (to-string-digit thingy)
            (let (
                (last-digit (- thingy (* 10 (floor thingy 10))))
                (rest (floor thingy 10)))
                (concatenate 'string (to-string-number rest) (to-string-digit last-digit))))))

(defun to-string-digit (thingy)
    (if (= thingy 0) "0"
    (if (= thingy 1) "1"
    (if (= thingy 2) "2"
    (if (= thingy 3) "3"
    (if (= thingy 4) "4"
    (if (= thingy 5) "5"
    (if (= thingy 6) "6"
    (if (= thingy 7) "7"
    (if (= thingy 8) "8"
    (if (= thingy 9) "9")))))))))))

(defun to-string-string (thingy)
    ;; TODO should escape interior quotes
    (concatenate 'string "\"" thingy "\""))

(defun to-string-symbol (thingy)
    (concatenate 'string "SYMBOL:" thingy))

(defun to-string-any (thingy)
    (if (null thingy)
        "NIL"
        (if (consp thingy)
            (to-string-list thingy)
            (if (numberp thingy)
                (to-string-number thingy)
                (if (stringp thingy)
                    (to-string-string thingy)
                    (if (symbolp thingy)
                        (to-string-symbol thingy)
                        (if (functionp thingy)
                            "FUNCTION"
                            "CANNOTDUMPTHIS")))))))

(defun dump (first &rest rest)
    (format t (to-string-any first))
    (if rest (apply #'dump rest))
    first)
