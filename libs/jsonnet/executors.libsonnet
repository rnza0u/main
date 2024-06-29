local localNodeExecutor = function(name){
    url: 'file://{{ root }}/executors/' + name,
    kind: 'Node',
    watch: [],
    rebuild: 'Always'
};

{
    cargoPublish: function() localNodeExecutor('cargo-publish'),
    npmPublish: function() localNodeExecutor('npm-publish'),
    pushTags: function() localNodeExecutor('push-tags'),
    npmVersionCheck: function() localNodeExecutor('npm-version-check'),
    cargoVersionCheck: function() localNodeExecutor('cargo-version-check'),
    packageBinaries: function() localNodeExecutor('package-binaries')
}