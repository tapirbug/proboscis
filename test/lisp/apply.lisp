(format t (apply #'concatenate '(strings "A" "b")))
(let ((c #'concatenate))
    (format t "cool")
    (format t (apply c '(strings "A" "b")))
    (format t "cool2"))

