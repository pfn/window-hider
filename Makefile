all: hide-app.exe

resource.o: resource.rc resource.h
	windres $< $@

.c.o: resource.h
	cc -Wall -c $<

hide-app.exe: hide-app.o resource.o
	cc -o $@ $^ -s -lgdi32 -lcomctl32 -Wl,--subsystem,windows

clean:
	rm *.o *.exe
