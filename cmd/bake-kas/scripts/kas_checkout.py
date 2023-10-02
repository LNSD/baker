import logging
import os
import sys

from kas import __version__
from kas.includehandler import IncludeHandler, IncludeException
from kas.libcmds import Loop, SetupReposStep, SetupDir, SetupHome,  FinishSetupRepos, \
    ReposApplyPatches, SetupEnviron, WriteBBConfig, Command
from kas.repos import Repo

try:
    import colorlog
    HAVE_COLORLOG = True
except ImportError:
    HAVE_COLORLOG = False


default_log_level = 'debug'


def create_logger():
    """
    Set up the logging environment
    """
    log = logging.getLogger()  # root logger
    log.setLevel(logging.getLevelName(default_log_level.upper()))
    format_str = '%(asctime)s - %(levelname)-8s - %(message)s'
    date_format = '%Y-%m-%d %H:%M:%S'
    if HAVE_COLORLOG and os.isatty(2):
        cformat = '%(log_color)s' + format_str
        colors = {'DEBUG': 'reset',
                  'INFO': 'reset',
                  'WARNING': 'bold_yellow',
                  'ERROR': 'bold_red',
                  'CRITICAL': 'bold_red'}
        formatter = colorlog.ColoredFormatter(cformat, date_format, log_colors=colors)
    else:
        formatter = logging.Formatter(format_str, date_format)
    stream_handler = logging.StreamHandler()
    stream_handler.setFormatter(formatter)
    log.addHandler(stream_handler)
    return logging.getLogger(__name__)


class Config:
    def __init__(self, filename, target=None, task=None, update=False):
        self._override_target = target
        self._override_task = task
        self._config = {}

        self.filenames = [os.path.abspath(configfile)
                          for configfile in filename.split(':')]
        top_repo_path = Repo.get_root_path(
            os.path.dirname(self.filenames[0]))
        repo_paths = [Repo.get_root_path(os.path.dirname(configfile),
                                         fallback=False)
                      for configfile in self.filenames]

        if len(set(repo_paths)) > 1:
            raise IncludeException('All concatenated config files must '
                                   'belong to the same repository or all '
                                   'must be outside of versioning control')

        self.handler = IncludeHandler(self.filenames,
                                      top_repo_path,
                                      not update)
        self.repo_dict = self._get_repo_dict()

    def get_build_system(self):
        """
            Returns the pre-selected build system
        """
        return self._config.get('build_system', '')

    def find_missing_repos(self, repo_paths={}):
        """
            Returns repos that are in config but not on disk
        """
        (self._config, missing_repo_names) = \
            self.handler.get_config(repos=repo_paths)

        return missing_repo_names

    def get_config(self):
        """
            Returns the config dict.
        """
        return self._config

    def get_repos_config(self):
        """
            Returns the repository configuration
        """
        return self._config.get('repos', {})

    def get_repos(self):
        """
            Returns the list of repos.
        """
        # Always keep repo_dict and repos synchronous
        # when calling get_repos
        self.repo_dict = self._get_repo_dict()
        return list(self.repo_dict.values())

    def get_repo(self, name):
        """
            Returns a `Repo` instance for the configuration with the key
            `name`.
        """

        repo_defaults = self._config.get('defaults', {}).get('repos', {})
        overrides = self._config.get('overrides', {}) \
            .get('repos', {}).get(name, {})
        config = self.get_repos_config()[name] or {}
        top_repo_path = self.handler.get_top_repo_path()
        return Repo.factory(name,
                            config,
                            repo_defaults,
                            top_repo_path,
                            overrides)

    def _get_repo_dict(self):
        """
            Returns a dictionary containing the repositories with
            their names (as it is defined in the config file) as keys
            and the `Repo` instances as values.
        """
        return {name: self.get_repo(name) for name in self.get_repos_config()}

    def get_bitbake_targets(self):
        """
            Returns a list of bitbake targets
        """
        if self._override_target:
            return self._override_target
        environ_targets = [i
                           for i in os.environ.get('KAS_TARGET', '').split()
                           if i]
        if environ_targets:
            return environ_targets
        target = self._config.get('target', 'core-image-minimal')
        if isinstance(target, str):
            return [target]
        return target

    def get_bitbake_task(self):
        """
            Returns the bitbake task
        """
        if self._override_task:
            return self._override_task
        return os.environ.get('KAS_TASK',
                              self._config.get('task', 'build'))

    def _get_conf_header(self, header_name):
        """
            Returns the local.conf header
        """
        header = ''
        for key, value in sorted(self._config.get(header_name, {}).items()):
            header += '# {}\n{}\n'.format(key, value)
        return header

    def get_bblayers_conf_header(self):
        """
            Returns the bblayers.conf header
        """
        return self._get_conf_header('bblayers_conf_header')

    def get_local_conf_header(self):
        """
            Returns the local.conf header
        """
        return self._get_conf_header('local_conf_header')

    def get_machine(self):
        """
            Returns the machine
        """
        return os.environ.get('KAS_MACHINE',
                              self._config.get('machine', 'qemux86-64'))

    def get_distro(self):
        """
            Returns the distro
        """
        return os.environ.get('KAS_DISTRO',
                              self._config.get('distro', 'poky'))

    def get_environment(self):
        """
            Returns the configured environment variables from the configuration
            file with possible overwritten values from the environment.
        """
        env = self._config.get('env', {})
        return {var: os.environ.get(var, env[var]) for var in env}

    def get_multiconfig(self):
        """
            Returns the multiconfig array as bitbake string
        """
        multiconfigs = set()
        for target in self.get_bitbake_targets():
            if target.startswith('multiconfig:') or target.startswith('mc:'):
                multiconfigs.add(target.split(':')[1])
        return ' '.join(multiconfigs)


class InitSetupRepos(Command):
    """
        Prepares setting up repos including the include logic
    """

    def __str__(self):
        return 'init_setup_repos'

    def execute(self, ctx):
        ctx.missing_repo_names = ctx.config.find_missing_repos()
        ctx.missing_repo_names_old = None


def kas_checkout(ctx, cfg, skip=None):
    # Set the context for the libkas module global variable
    import kas.context as libkas_context
    libkas_context.__context__ = ctx

    # Init logging and log kas version
    create_logger()

    logging.info('%s %s started', os.path.basename(sys.argv[0]), __version__)

    # Set up the tasks list
    repo_loop = Loop('repo_setup_loop')
    repo_loop.add(SetupReposStep())

    setup_commands = [
        SetupDir(),
        # SetupHome(),
        InitSetupRepos(),
        repo_loop,
        FinishSetupRepos(),
        ReposApplyPatches(),
        SetupEnviron(),
        WriteBBConfig(),
    ]

    # Execute the tasks
    for cmd in setup_commands:
        cmd_name = str(cmd)
        logging.info('execute %s', cmd_name)
        cmd.execute(ctx)