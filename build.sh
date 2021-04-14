if cargo build --release; then
	if [[ -z $1 || -z $2  || -z $3 ]]; then
	    echo ./build {vst name} {lib name} {local vst dir}
	else
		rm -fR ./$1.vst
		./bundle.sh $1 target/release/$2.dylib
		rm -fR $3/$1.vst
		cp -R ./$1.vst /Library/Audio/Plug-Ins/VST/Custom/$1.vst
	fi
else
	echo failed to compile
fi


