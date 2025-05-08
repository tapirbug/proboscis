
(defun testrest (first &rest rest)
    (concat-string-like-2 first (car rest)))

(format t (testrest "Schabracken" "tapir"))
