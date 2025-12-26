import init, {MidiPlayer } from "./pkg/dynamic_piano_sheet.js";
init().then((wasm) => {
  const canvas = document.getElementById('canvas');
  const ctx = canvas.getContext('2d');

  const midi_player = MidiPlayer.new();

  function resizeCanvas() {
      canvas.width = canvas.clientWidth;
      canvas.height = canvas.clientHeight;
  }

  window.addEventListener('load', resizeCanvas);
  window.addEventListener('resize', resizeCanvas);
  resizeCanvas();

  const loop_checkbox = document.getElementById("enable-loop");
  const loop_start_bar_input = document.getElementById("loop-start-bar");
  const loop_end_bar_input = document.getElementById("loop-end-bar");

  function update_loop_settings() {
    if (loop_checkbox.checked) {
      const start_bar = Math.max(1, Math.min(loop_start_bar_input.valueAsNumber, midi_player.num_bars()));
      const end_bar = Math.max(start_bar, Math.min(loop_end_bar_input.valueAsNumber, midi_player.num_bars()));
      midi_player.set_loop_bars(start_bar - 1, end_bar - 1);
    } else {
      midi_player.set_loop_bars(0, 0);
    }
  }

  loop_checkbox.addEventListener('change', (event) => {
    update_loop_settings();
  });

  loop_start_bar_input.addEventListener('input', (event) => {
    update_loop_settings();
  });

  loop_end_bar_input.addEventListener('input', (event) => {
    update_loop_settings();
  });

  const display_slider = document.getElementById("display-slider");
  display_slider.addEventListener('input', (event) => {
    midi_player.set_display_range(display_slider.valueAsNumber);
  });
  
  const bar_slider = document.getElementById("bar-slider");
  bar_slider.addEventListener('input', (event) => {
    const bar_number = bar_slider.valueAsNumber;
    midi_player.seek_bar(bar_number, true);
  });
  const bar_label = document.getElementById("bar-label");

  const volume_slider = document.getElementById("volume-slider");
  volume_slider.addEventListener('input', (event) => {
    midi_player.set_volume(volume_slider.valueAsNumber);
  });
  volume_slider.value = midi_player.volume();

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
    if( midi_player.ready())
      midi_player.play();
    else
      alert("MIDIファイルを選択してください");
  });

  const stop_button = document.getElementById("stop-button");
  stop_button.addEventListener('click', (event) => {
    midi_player.stop();
  });

  let canvasHold = false;
  canvas.onpointerdown = (e) =>{
      if(midi_player){
          e.preventDefault();
          canvasHold = true;
      }
  }

  canvas.onpointermove = (e) =>{
      if(canvasHold){
          e.preventDefault();
          midi_player.skip(e.movementY * display_slider.valueAsNumber / canvas.height);
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

  canvas.onwheel = (e) =>{
    e.preventDefault();
    midi_player.skip(e.deltaY * -1 * display_slider.valueAsNumber / canvas.height);
  }

  canvas.ondrop = (ev) => {
    ev.preventDefault();
    const dt = new DataTransfer();
    if (ev.dataTransfer.items) {
      [...ev.dataTransfer.items].forEach(async (item, i) => {
        if (item.kind === "file") {
          const file = item.getAsFile();
          load_midi(file);
          dt.items.add(file);
        }
      });
    } 
    else
    {
      [...ev.dataTransfer.files].forEach(async (file, i) => {
        load_midi(file);
        dt.items.add(file);
      });
    }
    midi_open.files = dt.files;
  }

  canvas.ondragover = (ev) => {
      ev.preventDefault();
  }

  let animationId = null;
  let lastTime = 0;
  const renderLoop = async (time) => {
    if (!lastTime)
      lastTime = time;
    
    // MIDIファイルの読み込み要求があればここで処理、読み込み中に割り込みでrenderloopが回るとmidi_playerが例外を発生することがあるので
    if (requested_midi_file !== null) {
      const file = requested_midi_file;
      requested_midi_file = null;
      await midi_player.load_midi(file).then(() =>{
        bar_slider.max = midi_player.num_bars() - 1;
        loop_start_bar_input.max = midi_player.num_bars();
        loop_end_bar_input.max = midi_player.num_bars();
      }).catch((err) => {
        alert("MIDIファイルの読み込みに失敗しました");
      });
    }

    const deltaTime = time - lastTime;
    lastTime = time;
    midi_player.tick(deltaTime);
    midi_player.render(ctx, 0, 0, canvas.width, canvas.height);

    bar_slider.value = midi_player.current_bar();
    bar_label.textContent = String(bar_slider.valueAsNumber + 1) + "/" + String(midi_player.num_bars());

    animationId = requestAnimationFrame(renderLoop);
  };

  renderLoop();
});
