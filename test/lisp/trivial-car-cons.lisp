(format t (concat-string-like-2 "tapir" (car '("stuff"))))
(format t (concat-string-like-2 "tapir" (car (cons "stuff" nil))))

(format t (concat-string-like-2 "tapir" "stuff2"))
