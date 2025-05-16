
(defun testrest (first &rest rest)
    (concatenate 'string first (car rest)))

(format t (testrest "Schabracken" "tapir"))
