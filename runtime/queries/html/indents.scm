; (
;   (start_tag) @start
;   (_)*
;   (_) @indent
;   (#not-same-line? @indent @start)
;   (#set! "scope" "all")
;   (end_tag)? @outdent
; )

(element) @indent
(end_tag) @outdent
(
  (start_tag) @pair.open
  (end_tag) @pair.close
)
