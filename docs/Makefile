
all: man1/ceres.1

man1/ceres.1: ceres.1.md man1 pandoc
	@pandoc -s -t man $< -o $@
	@sed -i.bk 's/\.TH.*/.TH "ceres" "1"/g' man1/ceres.1
	@rm -f man1/ceres.1.bk

man1:
	@mkdir $@

pandoc:
	@pandoc --version > /dev/null

view: man1/ceres.1
	@man -M . 1 ceres

clean:
	@-rm -R man1

.PHONY: pandoc view clean

