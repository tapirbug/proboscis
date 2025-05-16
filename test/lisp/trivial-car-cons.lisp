(format t (concatenate 'string "tapir" (car '("stuff"))))
(format t (concatenate 'string "tapir" (car (cons "stuff" nil))))

(format t (concatenate 'string "tapir" "stuff2"))
