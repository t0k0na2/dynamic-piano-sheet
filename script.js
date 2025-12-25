import init, {MidiPlayer } from "./pkg/dynamic_piano_sheet.js";
init().then((wasm) => {
  const canvas = document.getElementById('canvas');
  const ctx = canvas.getContext('2d');

  const midiPlayer = MidiPlayer.new();

  function resizeCanvas() {
      canvas.width = canvas.clientWidth;
      canvas.height = canvas.clientHeight;
  }

  window.addEventListener('load', resizeCanvas);
  window.addEventListener('resize', resizeCanvas);
  resizeCanvas();

  const display_slider = document.getElementById("display-slider");
  display_slider.addEventListener('input', (event) => {
    midiPlayer.set_display_range(display_slider.valueAsNumber);
  });
  
  const bar_slider = document.getElementById("bar-slider");
  bar_slider.addEventListener('input', (event) => {
    const bar_number = bar_slider.valueAsNumber;
    midiPlayer.seek_bar(bar_number, true);
  });
  const bar_label = document.getElementById("bar-label");

  const volume_slider = document.getElementById("volume-slider");
  volume_slider.addEventListener('input', (event) => {
    midiPlayer.set_volume(volume_slider.valueAsNumber);
  });
  volume_slider.value = midiPlayer.volume();

  let requested_midi_file = null;

  async function load_midi(file){
    requested_midi_file = file;
  }

  const midi_open = document.getElementById("midi-open");
  midi_open.addEventListener('change', async (event) => {
    const file = event.target.files[0];
    if (!file)
      return;
    load_midi(file);
  });

  const play_button = document.getElementById("play-button");
  play_button.addEventListener('click', (event) => {
    if( midiPlayer.ready())
      midiPlayer.play();
    else
      alert("MIDIファイルを選択してください");
  });

  const stop_button = document.getElementById("stop-button");
  stop_button.addEventListener('click', (event) => {
    midiPlayer.stop();
  });

  let canvasHold = false;
  canvas.onpointerdown = (e) =>{
      if(midiPlayer){
          e.preventDefault();
          canvasHold = true;
      }
  }

  canvas.onpointermove = (e) =>{
      if(canvasHold){
          e.preventDefault();
          midiPlayer.skip(e.movementY * display_slider.valueAsNumber / canvas.height);
      }
  }

  canvas.onpointerup = (e) =>{
      if(canvasHold){
          e.preventDefault();
          canvasHold = false;
      }
  }
  canvas.onpointercancel = (e) =>{
      if(canvasHold){
          e.preventDefault();
          canvasHold = false;
      }
  }

  canvas.ondrop = (ev) => {
    ev.preventDefault();

    if (ev.dataTransfer.items) {
      [...ev.dataTransfer.items].forEach(async (item, i) => {
        if (item.kind === "file") {
          const file = item.getAsFile();
          load_midi(file);
        }
      });
    } 
    else
    {
      [...ev.dataTransfer.files].forEach(async (file, i) => {
        load_midi(file);
      });
    }
  }

  canvas.ondragover = (ev) => {
      ev.preventDefault();
  }

  let animationId = null;
  let lastTime = 0;
  const renderLoop = async (time) => {
    if (!lastTime)
      lastTime = time;
    
    if (requested_midi_file !== null) {
      const file = requested_midi_file;
      requested_midi_file = null;
      await midiPlayer.load_midi(file).then(() =>{
        bar_slider.max = midiPlayer.num_bars() - 1;
      }).catch((err) => {
        alert("MIDIファイルの読み込みに失敗しました");
      });
    }

    const deltaTime = time - lastTime;
    lastTime = time;
    midiPlayer.tick(deltaTime);
    midiPlayer.render(ctx, 0, 0, canvas.width, canvas.height);

    bar_slider.value = midiPlayer.current_bar();
    bar_label.textContent = String(bar_slider.valueAsNumber + 1) + "/" + String(midiPlayer.num_bars());

    animationId = requestAnimationFrame(renderLoop);
  };

  renderLoop();
});
