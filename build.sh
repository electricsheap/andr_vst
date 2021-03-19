if cargo build --release; then
	if [[ -z $1 ]]; then
	    echo "Please give a name"
	else
		rm -fR ./$1.vst
		./bundle.sh $1 target/release/libconv.dylib
		rm -fR /Library/Audio/Plug-Ins/VST/Custom/$1.vst
		cp -R ./$1.vst /Library/Audio/Plug-Ins/VST/Custom/$1.vst
	fi
else
	echo failed to compile
fi


