(defun format (where format-str &rest args)
    ;; TODO actual formatting stuff
    (princ format-str))

(defun princ (string-or-symbol)
    ;; TODO: should check that the argument is a string or symbol
    (intrinsic:princ string-or-symbol))
