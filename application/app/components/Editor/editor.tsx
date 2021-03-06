import React, { useState, useEffect, useContext } from 'react';
import AceEditor from 'react-ace';
import WSCMode from './mode.js';
import './theme';
import { GlobalContext, Editors } from '../../store';
import { DispatchContext } from '../../actions/actions';

import styled from 'styled-components';

import 'ace-builds/src-noconflict/mode-elixir';
import 'ace-builds/src-noconflict/theme-github';
import 'ace-builds/src-noconflict/keybinding-vim';
import 'ace-builds/src-noconflict/keybinding-emacs';
import 'ace-builds/src-noconflict/ext-language_tools';

const customMode = new WSCMode();

type Props = { handleLoad: () => void };

const Container = styled.div`
  top: 140px;
  position: absolute;
  bottom: 23px;
  width: 80vw;
`;

export const Editor = (props: Props): React.ReactElement => {
  const store = useContext(GlobalContext);
  const dispatch = useContext(DispatchContext);

  const [renderSpace, setRenderSpace] = useState<AceEditor | null>();
  const [render, setRender] = useState<boolean>(false);
  const [save, setSave] = useState<boolean>(false);

  useEffect(() => {
    if (renderSpace) {
      const getLocalStorage = async () => {
        const storedLanguage = localStorage.getItem('language');
        const storedEditor = localStorage.getItem('editor');
        const storedVolume = localStorage.getItem('volume');
        const storedWorkingPath = localStorage.getItem('working_path');

        if (storedLanguage) {
          dispatch.onUpdateLanguage(storedLanguage);
        }
        if (storedWorkingPath) {
          dispatch.onSetWorkingPath(storedWorkingPath);
        }
        if (storedVolume) {
          await dispatch.onVolumeChange(parseInt(storedVolume));
        }
        if (storedEditor) {
          dispatch.onIncrementEditorType(parseInt(storedEditor) - 1);
        }
      };
      // @ts-ignore
      renderSpace.editor.getSession().setMode(customMode);
      renderSpace.editor.setTheme('ace/theme/wsc');
      getLocalStorage().catch((e) => {
        throw e;
      });
    }
  }, [renderSpace, dispatch]);

  useEffect(() => {
    const submit = async () => {
      if (render) {
        await dispatch.onRenderWithWorkingPath(
          store.language,
          store.working_path
        );
        setRender(false);
      }
    };

    submit().catch((e) => {
      throw e;
    });
  }, [render, dispatch, store.language, store.working_path]);

  useEffect(() => {
    if (save) {
      dispatch.onFileSave(store.language);
      setSave(false);
    }
  }, [store, dispatch, store.language, save]);

  useEffect(() => {
    if (store.initializeTest) {
      dispatch.onStop().catch((e) => {
        throw e;
      });
    }
  }, [dispatch, store.initializeTest]);

  const setRenderSpaceOuter = (el: AceEditor | null) => {
    if (el && !store.editor_ref) {
      dispatch.onSetEditorRef(el);
      setRenderSpace(el);
    }
  };

  return (
    <Container>
      <AceEditor
        focus={true}
        ref={(el) => {
          setRenderSpaceOuter(el);
        }}
        placeholder="WereSoCool"
        mode="elixir"
        theme="github"
        name="aceEditor"
        keyboardHandler={Editors[store.editor]['style']}
        value={store.language}
        onChange={(l) => dispatch.onUpdateLanguage(l)}
        markers={store.markers}
        fontSize={20}
        showPrintMargin={true}
        showGutter={true}
        highlightActiveLine={true}
        setOptions={{
          //enableLiveAutocompletion: true,
          enableBasicAutocompletion: true,
          showLineNumbers: true,
          tabSize: 2,
          displayIndentGuides: true,
        }}
        commands={[
          {
            name: 'submit',
            bindKey: { win: 'Shift-Enter', mac: 'Shift-Enter' },
            exec: () => {
              setRender(true);
            },
          },
          {
            name: 'stop',
            bindKey: { win: 'Ctrl-Enter', mac: 'Command-Enter' },
            exec: async () => {
              await dispatch.onStop();
            },
          },
          {
            name: 'save',
            bindKey: { win: 'Ctrl-s', mac: 'Command-s' },
            exec: () => {
              setSave(true);
            },
          },
          {
            name: 'load',
            bindKey: { win: 'Ctrl-l', mac: 'Command-l' },
            exec: () => {
              props.handleLoad();
            },
          },
        ]}
        style={{
          width: '100%',
          height: '100%',
          marginLeft: '10vw',
        }}
      />
    </Container>
  );
};
